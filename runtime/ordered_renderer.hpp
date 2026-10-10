// Included inside main.cpp's renderer namespace, after PolygonEmitter.
// Uses the same buffers, clipping implementation and frame accounting.
epok::OrderedDrawCursor<epok::ordered_draw_capacity> ordered_cursor;
epok::OrderedDrawStats ordered_work;
// Projection block offsets are valid for one epoch: a frame, or until a
// re-anchor moves the camera lattice. Never zero, the blocks' initial state.
uint32_t ordered_block_epoch=0;
inline uint32_t next_ordered_block_epoch(){
  ordered_block_epoch=(ordered_block_epoch+1)&0x7fffffffu;
  if(!ordered_block_epoch)ordered_block_epoch=1;
  return ordered_block_epoch;
}

#ifdef EPOK_VALIDATE_GTE
__attribute__((noinline,optimize("O3"))) void validate_ordered_gte(
    const Matrix& matrix,const epok::CameraSpace<Fixed>& view,const Fixed* row_scale,
    const int32_t (*local)[3],const int32_t* offset,const int32_t* base_offset,unsigned count,bool triple_valid,
    const uint32_t* screen,const int32_t* depth,uint32_t token) {
  auto& stats=epok::ordered_validation_stats;
  ++stats.triples;
  const auto fail=[&](uint32_t kind,unsigned v){
    ++performance_work.gte_validation_errors;++gte_validation_diagnostic[7];
    if(kind==1)++stats.camera_errors;else if(kind==2)++stats.batch_errors;else ++stats.screen_errors;
    if(!stats.first_error_kind){stats.first_error_kind=kind;stats.first_error_sample=ordered_cursor.sample();
      stats.first_error_token=token;stats.first_error_vertex=v;}
  };
  bool singles_valid=true;
  for(unsigned v=0;v<count;++v){
    int32_t camera[3],expected[3];uint32_t single_screen,flags;
    epok::project_geometry_vertex(local[v],offset,camera,single_screen,flags);
    // The oracle uses the ORIGINAL camera-relative origin, not the rebased
    // matrix/offset under test. This also checks the anchor algebra in hardware.
    epok::ordered_gte_camera_reference(matrix,local[v],base_offset,expected);
    ++stats.vertices;
    const bool safe=!(flags&epok::gte_projection_flags);
    stats.flagged_vertices+=!safe;if(v<3)singles_valid&=safe;
    for(unsigned c=0;c<3;++c){
      const int64_t difference=int64_t(camera[c])-expected[c];
      const uint32_t delta=uint32_t(difference<0?-difference:difference);
      if(delta>stats.max_camera_delta)stats.max_camera_delta=delta;
      if(delta)fail(1,v);
    }
    if(triple_valid&&(v<3||safe)&&(single_screen!=screen[v]||camera[2]!=depth[v]||!safe))fail(2,v);
    // The CPU fallback's GTE model must reproduce RTPS bit for bit wherever
    // the GTE accepted the vertex; any mismatch would reopen edge cracks.
    uint32_t modelled;
    if(safe&&!(epok::gte_project_point(expected,epok::display_height*2/3,epok::display_width/2,epok::display_height/2,modelled)
        &&modelled==single_screen))fail(5,v);
    if(safe&&camera[2]>=OrderedUnits::near&&camera[2]<OrderedUnits::far){
      ProjectedVertex reference{};for(unsigned c=0;c<3;++c)reference.camera[c]=camera[c];
      epok::project_cpu<OrderedUnits,epok::display_width,epok::display_height>(reference);
      const int dx=reference.screen.x-int16_t(single_screen),dy=reference.screen.y-int16_t(single_screen>>16);
      const uint32_t ax=uint32_t(dx<0?-dx:dx),ay=uint32_t(dy<0?-dy:dy),delta=ax>ay?ax:ay;
      if(delta>stats.max_screen_delta)stats.max_screen_delta=delta;
      if(delta>performance_work.gte_screen_max_delta)performance_work.gte_screen_max_delta=delta;
      if(delta>2)fail(3,v); // Quantized GTE reciprocal vs integer CPU division.
    }
    Fixed world[3],ideal[3];for(unsigned c=0;c<3;++c)world[c]=Fixed(local[v][c],Fixed::RAW);
    view.point(world,ideal);ideal[0]*=row_scale[0];ideal[1]*=row_scale[1];
    for(unsigned c=0;c<3;++c){
      const int64_t difference=int64_t(camera[c])*16-ideal[c].raw();
      const uint32_t delta=uint32_t(difference<0?-difference:difference);
      if(delta>stats.max_world_camera_delta)stats.max_world_camera_delta=delta;
    }
  }
  if(singles_valid!=triple_valid)fail(2,0); // Reject/accept agreement, not only matching accepted batches.
}
#endif

// Packet storage and the OT are read once per pass: every packet word is
// written through a uint32_t pointer, which would otherwise force the compiler
// to reload emitter fields and global counters after each store.
struct OrderedPacketTarget {
  uint32_t* gouraud;uint32_t* textured;decltype(&(static_cast<PolygonEmitter*>(nullptr)->table->m_table[0])) ot;
  uint32_t& next_dynamic;
};
// Returns the slots used: 0 when the frame's packet storage is exhausted.
template<bool Textured,bool Quad>
inline unsigned ordered_packet(const OrderedPacketTarget& target,const uint32_t* rgb,const uint32_t* screen,
    epok::BakedTextureWords uv,uint32_t command,int depth) {
  static_assert(sizeof(*static_cast<PolygonEmitter*>(nullptr)->gouraud)==28&&sizeof(*static_cast<PolygonEmitter*>(nullptr)->textured)==40);
  static_assert(epok::BakedPacket<Textured,Quad>::bytes_with_dma<=(Textured?40:28)*(Quad?2:1));
  constexpr unsigned slots=Quad?2:1;
  uint32_t slot;
  if constexpr(epok::retained_geometry)slot=Quad?frame_packet_slots.allocate_dynamic_pair():frame_packet_slots.allocate_dynamic();
  else {slot=target.next_dynamic>=slots?(target.next_dynamic-=slots):capacity;}
  if(slot>=capacity){epok::lighting_work.dropped_triangles+=slots;return 0;}
  // Both storage arrays contain contiguous DMA tag/payload words. A quad
  // borrows a verified adjacent pair, without constructing an aliasing object.
  auto* packet=Textured?target.textured+slot*10:target.gouraud+slot*7;
  epok::BakedPacket<Textured,Quad>::write(packet+1,rgb,screen,uv,command);
  auto& bucket=target.ot[depth+1];
  packet[0]=(epok::BakedPacket<Textured,Quad>::words<<24)|(bucket.head&0xffffffu);
  bucket.head=reinterpret_cast<uint32_t>(packet)&0xffffffu;
  return slots;
}

__attribute__((noinline,optimize("O3"))) void draw_ordered_world(
    PolygonEmitter& emitter,const epok::CameraSpace<Fixed>& view,const Fixed* row_scale) {
  if(!epok::visibility_track_started||!epok::active_visibility_track
      ||ordered_cursor.owner()!=epok::active_visibility_track->ordered
      ||ordered_cursor.sample()!=epok::visibility_page_selection.current)return;
  // Exact rendered view, not the mutable track state from the next frame.
  ordered_work.sample=ordered_cursor.sample();
  ordered_work.frame_microseconds=epok::time.frame_microseconds;
  for(unsigned c=0;c<3;++c)ordered_work.camera_origin[c]=view.origin[c].raw();
  for(unsigned r=0;r<3;++r)for(unsigned c=0;c<4;++c)
    ordered_work.camera_basis[r*4+c]=view.basis.values[r][c].raw();
  for(unsigned r=0;r<2;++r)ordered_work.row_scale[r]=row_scale[r].raw();
  Matrix matrix=view.basis;
  int32_t origin[3],offset[3],base_offset[3];Fixed residual[3],translation[3];
  for(unsigned c=0;c<3;++c){
    const auto q8=view.origin[c].raw()>>4;origin[c]=q8*16;base_offset[c]=offset[c]=-q8;
    residual[c]=Fixed(origin[c]-view.origin[c].raw(),Fixed::RAW);
  }
  matrix.point(residual,translation);
  for(unsigned r=0;r<3;++r)matrix.values[r][3]=translation[r];
  for(unsigned r=0;r<2;++r)for(unsigned c=0;c<4;++c)matrix.values[r][c]*=row_scale[r];
  const bool matrix_valid=epok::load_projection_matrix(matrix);
  if constexpr(epok::gte_depth_average)epok::load_average_depth_scale(epok::ordered_zsf3,epok::ordered_zsf4);
  bool anchored=false;
  uint32_t previous_page=0xffffffffu;const uint8_t* page=nullptr;
  // Resident pages do not move during a pass. Remember them in a small
  // direct-mapped table so a page switch rarely searches the pool; a slot
  // collision only costs a fresh search.
  struct {uint32_t id;const uint8_t* data;} page_cache[8];
  for(auto& entry:page_cache)entry={0xffffffffu,nullptr};
  uint32_t texture_key=0xffffffffu;int monotone_depth=0;
  bool textured=false;uint32_t clut16=0,tpage16=0;
  const OrderedPacketTarget target{reinterpret_cast<uint32_t*>(emitter.gouraud),
    reinterpret_cast<uint32_t*>(emitter.textured),emitter.table->m_table,emitter.next_dynamic};
  // Per-pass totals kept in locals and published once after the loop.
  // Vertex totals follow from the candidates processed and how many were
  // quads; drawn slots from packets and quads (a quad uses two). Counts of
  // at most 65535 (the cursor capacity) share a word, as low and high
  // halves, to keep fewer values live across the loop.
  uint32_t skipped=0,walk=0,cpu_vertices=0,anchor_base=0,anchor_misses=0; // skipped: candidates without a direct packet|their quads<<16; walk: quad candidates|page switches<<16
  const auto* baked_materials=epok::baked_texture_bank(epok::texture_assets);
  emitter.editable=false;emitter.double_sided=true;
  const auto* tokens=ordered_cursor.tokens();
  auto* projections=ordered_cursor.owner()->projections;
  uint32_t epoch=next_ordered_block_epoch();
  ordered_work.candidates=ordered_cursor.count();
  const size_t total=ordered_cursor.count();
  const uint32_t* cursor=tokens+total;
  while(cursor!=tokens){
    const auto token=*--cursor;const unsigned part=token&3;const bool quad=part==3;
    const uint32_t page_id=token>>13;
    if(page_id!=previous_page){
      walk+=1u<<16;
      previous_page=page_id;
      auto& cached=page_cache[page_id&7u];
      if(cached.id==page_id)page=cached.data;
      else{page=epok::stream_pool.ready_data(page_id);cached={page_id,page};}
      if(!page){++performance_work.stream_failed_chunks;epok::stream_failed=true;++cursor;break;}
    }
    walk+=quad;
    const auto& face=*reinterpret_cast<const epok::BakedDrawFace*>(page+((token&8191u)&~3u)*8);
    const uint32_t block_id=epok::ordered_face_block(face.topology);
    auto& block=projections[block_id];
    // Projected corner v is face corner v, or v+1 for a quad's second half
    // (1,2,3); the quad's fourth is corner 3. Computed, not an array, so no
    // per-face table is stored for the variable-index uses.
    const unsigned first=part==2;
    const unsigned count=quad?4:3;
    epok::GteProjectionInput fourth;
    // Exact Q12 corners, decoded only for the per-vertex, CPU and validation
    // paths. world[v] is corner first+v; the quad's fourth is corner 3.
    int32_t world[4][3];bool decoded=false;
    const auto decode=[&]{
      if(decoded)return;
      for(unsigned v=0;v<count;++v)epok::ordered_vertex_world(block,page+face.indices[first+v],world[v]);
      decoded=true;
    };
    const auto prepare_inputs=[&](const int32_t* candidate_offset){
      decode();
      // Bitwise combination is intentional: every vector register must be
      // loaded, even when an earlier corner needs the wide CPU fallback.
      bool fits=epok::load_projection_input<0>(world[0],candidate_offset);
      fits&=epok::load_projection_input<1>(world[1],candidate_offset);
      fits&=epok::load_projection_input<2>(world[2],candidate_offset);
      if(quad)fits&=epok::pack_projection_input(world[3],candidate_offset,fourth);
      return fits;
    };
    EPOK_DETAIL_BEGIN(ordered_vertex);
    if((block.state>>1)!=epoch){
      // One bounds check per block and epoch replaces a range check per
      // vertex. Inputs equal (Q12 >> 4) + offset exactly, as per vertex.
      // An unusable matrix marks every block as not fitting, so the hot
      // test below needs only the block state.
      bool fits=((block.flags&epok::ordered_projection_compact)!=0)&matrix_valid;
      for(unsigned c=0;c<3;++c){
        block.sum[c]=block.center[c]+offset[c];
        fits&=(block.sum[c]+block.low[c]>=-32768)&(block.sum[c]+block.high[c]<=32767);
      }
      block.xy=uint32_t(uint16_t(block.sum[0]))|(uint32_t(block.sum[1])<<16);
      block.state=(epoch<<1)|uint32_t(fits);
    }
    bool narrow;
    if(block.state&1){
      const epok::CompactProjectionOffset compact{block.xy,uint32_t(uint16_t(block.sum[0])),uint32_t(block.sum[2])};
      epok::load_compact_projection_input<0>(page+face.indices[first],compact);
      epok::load_compact_projection_input<1>(page+face.indices[first+1],compact);
      epok::load_compact_projection_input<2>(page+face.indices[first+2],compact);
      if(quad)fourth=epok::compact_projection_input(page+face.indices[3],compact);
      narrow=true;
    }else if((block.flags&epok::ordered_projection_compact)&&matrix_valid){
      // Part of the block is out of range: check this face's corners only.
      epok::GteProjectionInput input[3];
      bool fits=epok::checked_compact_projection_input(page+face.indices[first],block.sum,input[0]);
      fits&=epok::checked_compact_projection_input(page+face.indices[first+1],block.sum,input[1]);
      fits&=epok::checked_compact_projection_input(page+face.indices[first+2],block.sum,input[2]);
      if(quad)fits&=epok::checked_compact_projection_input(page+face.indices[3],block.sum,fourth);
      epok::load_projection_words<0>(input[0]);
      epok::load_projection_words<1>(input[1]);
      epok::load_projection_words<2>(input[2]);
      narrow=fits;
    }else narrow=prepare_inputs(offset)&matrix_valid;
#ifdef EPOK_VALIDATE_GTE
    decode();
    const auto& local=world;
#endif
    if(!narrow&&matrix_valid){
      int32_t next_offset[3],next_translation[3];
      if(epok::ordered_projection_anchor(matrix,base_offset,block.center,next_offset,next_translation)){
        const bool fits=prepare_inputs(next_offset);
        if(fits){
          for(unsigned c=0;c<3;++c)offset[c]=next_offset[c];
          epok::load_projection_translation(next_translation);
          epoch=next_ordered_block_epoch();
          // Vertices before this candidate, for the anchored total below.
          if(!anchored)anchor_base=uint32_t(tokens+total-cursor-1)*3+(walk&0xffffu)-quad;
          ++ordered_work.origin_changes;anchored=true;narrow=true;
        }else ++ordered_work.anchor_rejections;
      }else ++ordered_work.anchor_rejections;
    }
    // Successful RTPT writes all three outputs; RTPS writes the fourth quad
    // output. A rejected/too-wide batch takes the CPU path, which writes every
    // used corner even when clipping is required. Never read an unused fourth
    // triangle corner. Clearing these arrays per face is therefore redundant.
    uint32_t screen[4];int32_t depth[4];
    if(narrow)epok::begin_loaded_projection_triple();
    // Without exact near clipping a flagged (saturated) GTE result is drawn as
    // is, like the reference renderer; triple_valid keeps the flag verdict.
    bool triple_valid,direct;
    if constexpr(epok::exact_near_clipping){
      triple_valid=narrow&&epok::finish_projection_triple(screen,depth);
      direct=triple_valid;
    }else{
      triple_valid=narrow&&epok::store_projection_triple(screen,depth);
      direct=narrow;
    }
    // A quad's fourth corner: issue RTPS, then test the first three corners
    // against the viewport while the GTE computes it.
    using ScreenSize=epok::OrderedScreenInside<epok::display_width,epok::display_height>;
    uint32_t inside_bits=0;
    if(direct){
      if(quad)epok::begin_packed_vertex(fourth);
      inside_bits=epok::ordered_screen_inside_bits<epok::display_width,epok::display_height>(screen[0])
        |epok::ordered_screen_inside_bits<epok::display_width,epok::display_height>(screen[1])
        |epok::ordered_screen_inside_bits<epok::display_width,epok::display_height>(screen[2]);
      if(quad){
        const bool fourth_valid=epok::finish_packed_vertex(screen[3],depth[3]);
        if constexpr(epok::exact_near_clipping)direct=fourth_valid;
        inside_bits|=epok::ordered_screen_inside_bits<epok::display_width,epok::display_height>(screen[3]);
      }
    }
    // Read while the SZ FIFO still holds this face (validation re-projects).
    int32_t gte_average=0;
    if constexpr(epok::gte_depth_average)if(direct)gte_average=epok::gte_average_depth(quad);
#ifdef EPOK_VALIDATE_GTE
    if(narrow)validate_ordered_gte(matrix,view,row_scale,local,offset,base_offset,count,triple_valid,screen,depth,token);
    else epok::ordered_validation_stats.range_fallback_vertices+=count;
#endif
    int min_x=32767,max_x=-32768,min_y=32767,max_y=-32768;
    bool inside=direct&&!(inside_bits&ScreenSize::mask);
    // Early trivial rejection, as the reference pass does: a GTE-projected
    // face entirely beside the viewport skips the bounds, clipping and
    // material work. Tested only for faces not already accepted, so faces
    // fully inside (the common case) pay nothing for it. Exact near clipping
    // keeps a face with a corner nearer than the plane for its clipping path;
    // every other rejected face is one the bounds cull below would drop (the
    // inexact path's too-wide skip drops it too).
    if(direct&&!inside&&epok::ordered_screen_outside<epok::display_width,epok::display_height>(screen,quad)){
      bool reject=true;
      if constexpr(epok::exact_near_clipping){
        reject=(depth[0]>=OrderedUnits::near)&(depth[1]>=OrderedUnits::near)&(depth[2]>=OrderedUnits::near);
        if(quad)reject&=depth[3]>=OrderedUnits::near;
      }
      if(reject){
        EPOK_DETAIL_END(ordered_vertex,vertex_scanlines);skipped+=1u+(uint32_t(quad)<<16);continue;
      }
    }
    if(direct){
      if(inside){
        if constexpr(epok::exact_near_clipping){
          direct=(depth[0]>=OrderedUnits::near)&(depth[1]>=OrderedUnits::near)&(depth[2]>=OrderedUnits::near);
          if(quad)direct&=depth[3]>=OrderedUnits::near;
        }
      }else{
        for(unsigned v=0;v<count;++v){
          const int x=int16_t(screen[v]),y=int16_t(screen[v]>>16);
          if(x<min_x)min_x=x;if(x>max_x)max_x=x;if(y<min_y)min_y=y;if(y>max_y)max_y=y;
          if constexpr(epok::exact_near_clipping)direct&=depth[v]>=OrderedUnits::near;
        }
        const bool fits_gpu=max_x-min_x<=1023&&max_y-min_y<=511;
        // The GPU itself skips a primitive wider than this; the reference
        // pass leaves that to the GPU, so the inexact path skips it here.
        if constexpr(!epok::exact_near_clipping)if(!fits_gpu){
          ++ordered_work.clipped_candidates;
          EPOK_DETAIL_END(ordered_vertex,vertex_scanlines);skipped+=1u+(uint32_t(quad)<<16);continue;
        }
        direct&=fits_gpu;
      }
    }
    // Written by the CPU projection before any read; raw storage avoids
    // default-initializing four vertices for every GTE-projected face.
    alignas(ProjectedVertex) unsigned char projected_storage[sizeof(ProjectedVertex)*4];
    auto* projected=reinterpret_cast<ProjectedVertex*>(projected_storage);
    const bool gte_projected=direct;
    if(!direct){
      inside=false;
      // Inexact clipping only reaches here for inputs too wide for the GTE.
      decode();
      direct=epok::project_ordered_cpu<OrderedUnits,epok::display_width,epok::display_height>(
        world,count,view,row_scale,projected,screen,depth,narrow?&matrix:nullptr,base_offset);
      ordered_work.cpu_direct+=direct;
      min_x=min_y=32767;max_x=max_y=-32768;
      if(direct)for(unsigned v=0;v<count;++v){
        const int x=int16_t(screen[v]),y=int16_t(screen[v]>>16);
        if(x<min_x)min_x=x;if(x>max_x)max_x=x;if(y<min_y)min_y=y;if(y>max_y)max_y=y;
      }
      cpu_vertices+=count;
      if(!narrow&&anchored)anchor_misses+=count;
    }
    EPOK_DETAIL_END(ordered_vertex,vertex_scanlines);
    EPOK_DETAIL_BEGIN(ordered_polygon);
    if(direct&&!inside&&(max_x<0||min_x>=epok::display_width||max_y<0||min_y>=epok::display_height)){
      EPOK_DETAIL_END(ordered_polygon,polygon_scanlines);skipped+=1u+(uint32_t(quad)<<16);continue;
    }
    // PsyQo traverses buckets DESCENDING. Inverting the source's ascending
    // min-cursor policy gives a max cursor, preserving the prepared order.
    // Depth units remain Epok's so ordinary dynamic actors share this OT.
    // Source uses AVSZ3 for triangles and AVSZ4 for full quads (800419bc).
    // Q8 camera values originate in signed Q12, so the four-term sum fits i32.
    int32_t average;
    if constexpr(epok::gte_depth_average){
      if(gte_projected)average=gte_average;
      else average=epok::ordered_gte_depth_average(depth[0]+depth[1]+depth[2]+(quad?depth[3]:0),quad);
#ifdef EPOK_VALIDATE_GTE
      if(gte_projected&&average!=epok::ordered_gte_depth_average(depth[0]+depth[1]+depth[2]+(quad?depth[3]:0),quad)){
        ++performance_work.gte_validation_errors;++epok::ordered_validation_stats.batch_errors;
        if(!epok::ordered_validation_stats.first_error_kind){epok::ordered_validation_stats.first_error_kind=4;
          epok::ordered_validation_stats.first_error_sample=ordered_cursor.sample();epok::ordered_validation_stats.first_error_token=token;}
      }
#endif
    }else average=epok::ordered_depth_average(depth[0]+depth[1]+depth[2]+(quad?depth[3]:0),quad);
    int candidate=average+face.depth_bias;
    if(candidate<0)candidate=0;if(candidate>=buckets)candidate=buckets-1;
    if(candidate>monotone_depth)monotone_depth=candidate;
    const uint32_t key=uint32_t(face.texture)|(uint32_t((face.flags>>1)&7)<<16);
    if(key!=texture_key){
      texture_key=key;
      if(baked_materials){
        const auto packet=epok::baked_texture_packet<epok::display_interlaced,epok::dither_3d>(baked_materials,epok::texture_count,face.texture,epok::BlendMode((face.flags>>1)&7));
        textured=packet.textured;clut16=packet.clut16;tpage16=packet.tpage16;
      }else{
        const auto* texture=epok::texture(face.texture==0xffff?-1:face.texture);textured=texture!=nullptr;
        if(texture){auto clut=epok::texture_clut(*texture);auto tpage=epok::texture_page(*texture,epok::BlendMode((face.flags>>1)&7),epok::dither_3d);
          uint16_t cb,pb;__builtin_memcpy(&cb,&clut,2);__builtin_memcpy(&pb,&tpage,2);clut16=uint32_t(cb)<<16;tpage16=uint32_t(pb)<<16;}
      }
    }
    const uint32_t command=epok::baked_command(face);
    epok::BakedTextureWords uv{clut16|face.uv[0],tpage16|face.uv[1],uint32_t(face.uv[2])|(uint32_t(face.uv[3])<<16)};
    if(part==2)uv=epok::baked_second_half(uv);
    if(direct){
      // Decoded here, not shared with the clipped branch, so the corner colors
      // never have their address taken and can stay in registers.
      uint32_t rgb[4];epok::baked_ordered_colors(face,part==2,epok::vertex_color_palette,rgb);
      unsigned slots;
      if(textured){if(quad)slots=ordered_packet<true,true>(target,rgb,screen,uv,command,monotone_depth);
        else slots=ordered_packet<true,false>(target,rgb,screen,uv,command,monotone_depth);}
      else {if(quad)slots=ordered_packet<false,true>(target,rgb,screen,uv,command,monotone_depth);
        else slots=ordered_packet<false,false>(target,rgb,screen,uv,command,monotone_depth);}
      if(!slots)skipped+=1u+(uint32_t(quad)<<16);
    }else{
      ++ordered_work.clipped_candidates;skipped+=1u+(uint32_t(quad)<<16);
      if constexpr(!epok::exact_near_clipping){EPOK_DETAIL_END(ordered_polygon,polygon_scanlines);continue;}
      uint32_t rgb[4];epok::baked_ordered_colors(face,part==2,epok::vertex_color_palette,rgb);
      QuadPacket material;
      material.textured=textured;material.clut16=clut16;material.tpage16=tpage16;
      material.command=command;material.gpu_colors=epok::baked_indexed_color(face);material.depth_bias=face.depth_bias;
      emitter.fixed_depth=monotone_depth;
      uint32_t shaded[4];for(unsigned v=0;v<4;++v)shaded[v]=material.gpu_colors?rgb[v]:face.shaded;
      if(quad)emitter.clipped<OrderedUnits,true>(projected[1],projected[2],projected[3],shaded[1],shaded[2],shaded[3],face.uv[1],face.uv[2],face.uv[3],material);
      emitter.clipped<OrderedUnits,true>(projected[0],projected[1],projected[2],shaded[0],shaded[1],shaded[2],face.uv[first],face.uv[first+1],face.uv[first+2],material);
      emitter.fixed_depth=-1;
    }
    EPOK_DETAIL_END(ordered_polygon,polygon_scanlines);
  }
  emitter.fixed_depth=-1;
  // Every processed candidate either drew one direct packet or was skipped.
  const uint32_t drawn=uint32_t(tokens+total-cursor)+((walk&0xffffu)<<16)-skipped;
  const uint32_t packets=drawn&0xffffu,quads=drawn>>16;
  emitter.emitted+=packets+quads;
  ordered_work.packets+=packets;ordered_work.quads+=quads;ordered_work.page_switches+=walk>>16;
  // The cursor stops at the first candidate not processed (the failed one
  // after a stream break); each processed candidate had 3 or 4 vertices.
  const uint32_t transformed=uint32_t(tokens+total-cursor)*3+(walk&0xffffu);
  // Anchored vertices: those of every GTE-range (narrow) candidate from the
  // first re-anchor on, i.e. all since then less the CPU-range ones.
  if(anchored)ordered_work.anchored_vertices+=transformed-anchor_base-anchor_misses;
  performance_work.software_vertices+=cpu_vertices;
  performance_work.gte_vertices+=transformed-cpu_vertices;epok::mesh_stats.transformed_vertices+=transformed;
}
