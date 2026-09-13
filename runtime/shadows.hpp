#pragma once
#include "lighting.hpp"
#include "psyqo/fragments.hh"
#include "psyqo/primitives/triangles.hh"

namespace epok {
class BlobRenderer {
    // Each packet restores blend state, so painter ordering cannot leak it into HUD/materials.
    struct Packet {uint32_t subtract=0xe1000240;psyqo::Prim::GouraudTriangle triangle;uint32_t restore=0xe1000200;};
    psyqo::Fragments::SimpleFragment<Packet> packets[2][256];
public:
    template<size_t N,typename Table>void draw(int parity,Table& table,const std::array<Entity,N>& objects,const std::array<Affine<Fixed>,N>& world,size_t count,const Affine<Fixed>& view){
        using namespace lighting_detail;size_t used=0,blobs=0;
        for(size_t i=0;i<count && blobs<32;++i){const auto& b=objects[i].blob_shadow;if(!b.enabled||!is_active_slot(i))continue;
            int center[3];for(int c=0;c<3;++c)center[c]=world[i].values[c][3].raw();int highest=INT32_MIN;int lower[3]={},upper[3]={};
            int distance=clamp(b.distance.raw(),1,32*4096);
            for(size_t j=0;j<count;++j){if(j==i || !objects[j].mesh || !objects[j].tiled || !is_active_slot(j))continue;const auto& m=world[j];bool aligned=true;for(int r=0;r<3;++r)for(int c=0;c<3;++c)if(r!=c && (m.values[r][c].raw()>1 || m.values[r][c].raw()<-1))aligned=false;if(!aligned)continue;
                int min[3],max[3];for(int c=0;c<3;++c){min[c]=m.values[c][3].raw()-m.values[c][c].raw()/2;max[c]=m.values[c][3].raw()+m.values[c][c].raw()/2;}
                if(max[1]>center[1] || center[1]-max[1]>distance || max[1]<=highest || center[0]<min[0] || center[0]>max[0] || center[2]<min[2] || center[2]>max[2])continue;
                highest=max[1];for(int c=0;c<3;++c){lower[c]=min[c];upper[c]=max[c];}
            }
            if(highest==INT32_MIN)continue;++blobs;
            int strength=clamp(b.strength.raw(),0,4096);strength=strength*((distance-(center[1]-highest))*4096/distance)/4096;
            const auto color=psyqo::Color{{.r=uint8_t(strength*255/4096),.g=uint8_t(strength*255/4096),.b=uint8_t(strength*255/4096)}};
            constexpr int circle[8][2]={{4096,0},{2896,2896},{0,4096},{-2896,2896},{-4096,0},{-2896,-2896},{0,-4096},{2896,-2896}};
            psyqo::Vertex screen[9];int depths[9];bool visible[9];int radius=clamp(b.radius.raw(),41,8*4096);
            for(int v=0;v<9;++v){Fixed p[3]={Fixed(center[0],Fixed::RAW),Fixed(highest+25,Fixed::RAW),Fixed(center[2],Fixed::RAW)};
                if(v){p[0]=Fixed(clamp(center[0]+circle[v-1][0]*radius/4096,lower[0],upper[0]),Fixed::RAW);p[2]=Fixed(clamp(center[2]+circle[v-1][1]*radius/4096,lower[2],upper[2]),Fixed::RAW);}
                Fixed projected[3];view.point(p,projected);projected[0]*=projection_focal;projected[1]*=projection_focal;int z=projected[2].raw();depths[v]=z;visible[v]=z>=1024 && z<128*4096;if(!visible[v])continue;
                int x=display_width/2+(projected[0].raw()/16)*(display_width/2)/(z/16),y=display_height/2-(projected[1].raw()/16)*(display_height*2/3)/(z/16);visible[v]=x>=-1023 && x<=1023 && y>=-1023 && y<=1023;screen[v]={{.x=int16_t(x),.y=int16_t(y)}};
            }
            for(int v=1;v<=8;++v){int next=v==8?1:v+1;if(!visible[0] || !visible[v] || !visible[next])continue;
                int depth=(depths[0]+depths[v]+depths[next])/(3*1024)-1;if(depth<0 || depth>=512)continue;
                auto& p=packets[parity][used++];p.primitive.triangle.setPointA(screen[0]).setPointB(screen[v]).setPointC(screen[next]).setColorA(color).setColorB(psyqo::Color{.packed=0}).setColorC(psyqo::Color{.packed=0}).setSemiTrans();table.insert(p,depth);
            }
        }lighting_work.blob_triangles=used;
    }
};
}
