"""Generate Epok's original rigid mannequin (Blender 5.1, no third-party assets).
Run: blender --background --factory-startup --python tests/fixtures/create_skeletal_fixture.py
The FBX is checked in; Blender is not required by the importer or test suite.
"""
import bpy, math
from pathlib import Path
bpy.ops.object.select_all(action='SELECT')
bpy.ops.object.delete(use_global=False)
rig_data=bpy.data.armatures.new('EpokRig')
rig=bpy.data.objects.new('EpokMannequin',rig_data)
bpy.context.collection.objects.link(rig)
bpy.context.view_layer.objects.active=rig
rig.select_set(True)
bpy.ops.object.mode_set(mode='EDIT')
specs=[
 ('Root',(0,0,0),(0,0,.2),None),
 ('Pelvis',(0,0,.9),(0,0,1.05),'Root'),
 ('Spine',(0,0,1.05),(0,0,1.5),'Pelvis'),
 ('Head',(0,0,1.5),(0,0,1.85),'Spine'),
 ('Arm.L',(.34,0,1.45),(.34,0,.95),'Spine'),
 ('Arm.R',(-.34,0,1.45),(-.34,0,.95),'Spine'),
 ('Thigh.L',(.15,0,.95),(.15,0,.52),'Pelvis'),
 ('Calf.L',(.15,0,.52),(.15,0,.10),'Thigh.L'),
 ('Thigh.R',(-.15,0,.95),(-.15,0,.52),'Pelvis'),
 ('Calf.R',(-.15,0,.52),(-.15,0,.10),'Thigh.R'),
]
for name,head,tail,parent in specs:
 b=rig_data.edit_bones.new(name);b.head=head;b.tail=tail
 if parent:b.parent=rig_data.edit_bones[parent]
bpy.ops.object.mode_set(mode='OBJECT')
materials=[]
for name,color in [('Armor',(.12,.42,.78,1)),('Joints',(.12,.15,.2,1)),('Face',(.8,.87,.92,1))]:
 m=bpy.data.materials.new(name);m.diffuse_color=color;m.use_nodes=True
 m.node_tree.nodes.get('Principled BSDF').inputs['Base Color'].default_value=color
 materials.append(m)
vertices=[];faces=[];groups=[];slots=[]
def box(center,size,bone,slot):
 n=len(vertices)
 vertices.extend(tuple(center[i]+sign[i]*size[i]/2 for i in range(3)) for sign in [(-1,-1,-1),(1,-1,-1),(1,1,-1),(-1,1,-1),(-1,-1,1),(1,-1,1),(1,1,1),(-1,1,1)])
 faces.extend(tuple(n+i for i in f) for f in [(0,3,2,1),(4,5,6,7),(0,1,5,4),(3,7,6,2),(0,4,7,3),(1,2,6,5)])
 groups.append((bone,list(range(n,n+8))));slots.extend([slot]*6)
box((0,0,1.26),(.53,.29,.42),'Spine',0)
box((0,0,1.),(.43,.25,.18),'Pelvis',1)
box((0,-.01,1.67),(.32,.3,.31),'Head',2)
box((0,-.168,1.7),(.25,.025,.075),'Head',1)
for side,x in [('L',.15),('R',-.15)]:
 box((x,0,.74),(.19,.22,.37),'Thigh.'+side,0)
 box((x,0,.31),(.16,.18,.37),'Calf.'+side,2)
 box((x,-.055,.09),(.2,.32,.16),'Calf.'+side,1)
 box((.35 if side=='L' else -.35,0,1.21),(.15,.2,.43),'Arm.'+side,0)
mesh=bpy.data.meshes.new('MannequinMesh');mesh.from_pydata(vertices,[],faces);mesh.update()
obj=bpy.data.objects.new('MannequinMesh',mesh);bpy.context.collection.objects.link(obj)
for mat in materials:mesh.materials.append(mat)
for poly,slot in zip(mesh.polygons,slots):poly.material_index=slot
for name,ids in groups:
 group=obj.vertex_groups.get(name) or obj.vertex_groups.new(name=name);group.add(ids,1.,'REPLACE')
modifier=obj.modifiers.new('Armature','ARMATURE');modifier.object=rig;obj.parent=rig
for name in ['Idle','Walk']:
 rig.animation_data_clear();rig.animation_data_create();action=bpy.data.actions.new(name);action.use_fake_user=True;rig.animation_data.action=action
 for frame in range(1,62,5):
  phase=2*math.pi*(frame-1)/60
  for bone in rig.pose.bones:
   bone.rotation_mode='XYZ';bone.rotation_euler=(0,0,0);bone.location=(0,0,0)
   if name=='Walk':
    if bone.name.startswith('Thigh'):bone.rotation_euler.x=.55*math.sin(phase+(math.pi if bone.name.endswith('R') else 0))
    if bone.name.startswith('Calf'):bone.rotation_euler.x=-.65*max(0,math.sin(phase+(math.pi if bone.name.endswith('R') else 0)))
    if bone.name.startswith('Arm'):bone.rotation_euler.x=.4*math.sin(phase+(math.pi if bone.name.endswith('L') else 0))
    if bone.name=='Pelvis':bone.location.y=.025*math.cos(2*phase)
   else:
    if bone.name=='Spine':bone.rotation_euler.z=.025*math.sin(phase)
    if bone.name=='Head':bone.rotation_euler.y=.08*math.sin(phase)
   bone.keyframe_insert('rotation_euler',frame=frame);bone.keyframe_insert('location',frame=frame)
 rig.animation_data.action=None
bpy.context.scene.render.fps=30
bpy.context.scene.frame_start=1;bpy.context.scene.frame_end=61
bpy.ops.object.select_all(action='DESELECT');rig.select_set(True);obj.select_set(True)
path=Path(__file__).resolve().parents[2]/'resources/models/EpokMannequin.fbx';path.parent.mkdir(parents=True,exist_ok=True)
bpy.ops.export_scene.fbx(filepath=str(path),use_selection=True,object_types={'ARMATURE','MESH'},add_leaf_bones=False,bake_anim=True,bake_anim_use_nla_strips=False,bake_anim_use_all_actions=True,bake_anim_force_startend_keying=True,bake_anim_simplify_factor=0.,axis_forward='-Z',axis_up='Y')
print('Saved',path)
