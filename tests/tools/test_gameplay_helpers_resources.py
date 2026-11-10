"""Focused semantic parity checks. Runs against unapplied draft by default here."""
import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import sys
import tempfile
import unittest
import uuid

HERE=Path(__file__).resolve().parent
ROOT=next(parent for parent in HERE.parents if (parent/'tools/epok_documents.py').is_file())
sys.path.insert(0,str(ROOT/'tools'))
import epok_documents as documents
target=HERE/'gameplay_helpers_acceptance.py'
if not target.is_file():target=ROOT/'tools/gameplay_helpers_acceptance.py'
spec=importlib.util.spec_from_file_location('resource_acceptance_under_test',target)
a=importlib.util.module_from_spec(spec);spec.loader.exec_module(a)


class SharedResourcesTests(unittest.TestCase):
    def setUp(self):
        output=ROOT/'artifacts/gameplay-helpers/p10/resource-parity/host-fixtures'
        output.mkdir(parents=True,exist_ok=True)
        self.root=Path(tempfile.mkdtemp(prefix=self._testMethodName+'-',dir=output))
        self.projects={};self.roles={};self.ids={}
        for flavor in ['cpp','blueprint','lua']:
            project=self.root/flavor;self.projects[flavor]=project
            ids={name:str(uuid.uuid5(uuid.NAMESPACE_URL,flavor+'/'+name)) for name in ['class','textureA','textureB','tiles','recipe','root','child','rect','image','text','childrect','generator']}
            self.ids[flavor]=ids;self.roles[flavor]={ids['class']:'Widget','Widget':'Widget'}
            self.package(flavor,'A',b'pixels A',asset_id=ids['textureA'],kind='Texture')
            self.package(flavor,'B',b'pixels B',asset_id=ids['textureB'],kind='Texture')
            self.package(flavor,'Tiles',{'texture':ids['textureA'],'tiles':[{'collision':'Full','probability':1.0}]},asset_id=ids['tiles'],kind='TileSet')
            self.package(flavor,'Recipe',{'steps':[{'step':'Lattice','size':[6,5]}]},asset_id=ids['recipe'],kind='DungeonRecipe')
            actors=[{'id':ids['root'],'name':'Root','class':{'name':'Widget','class_id':ids['class']},'components':[{'id':ids['rect'],'class':{'name':'Rect','class_id':'sdk-rect'},'properties':{'position':[0,0]}},{'id':ids['image'],'class':{'name':'Image','class_id':'sdk-image'},'properties':{'texture':ids['textureA']}}],'properties':{'child':ids['child']}},
                    {'id':ids['child'],'name':'Child','class':{'name':'UIActor','class_id':'sdk-ui'},'logical_parent':ids['root'],'attach':{'actor':ids['root']},'components':[{'id':ids['text'],'class':{'name':'Text','class_id':'sdk-text'},'properties':{'text':'Editable'}},{'id':ids['childrect'],'class':{'name':'Rect','class_id':'sdk-rect'},'properties':{'position':[12,-12]}}]}]
            scene={'actors':copy.deepcopy(actors),'bake':{'fingerprint':len(flavor),'enabled':True},'dungeons':[{'id':ids['generator'],'recipe':ids['recipe'],'seed':42,'output':{'fingerprint':flavor+'-cache','recipe':flavor+'-recipe','assets':[ids['textureA']]}}]}
            self.doc(flavor,'assets/scenes/Main.epokmap',scene)
            self.doc(flavor,'assets/scenes/Other.epokmap',{'actors':[],'ambient':[1,2,3]})
            template={'actors':[{'entity':actors[0],'parent':None},{'entity':actors[1],'parent':ids['root']}],'overrides':{},'references':[{'owner':ids['root'],'member':'child','target':ids['child']}],'construction':[]}
            if flavor=='blueprint':self.doc(flavor,'assets/Blueprints/Widget.epokbp',{'id':ids['class'],'name':'Widget','template':template,'functions':[{'body':'selected Blueprint rules'}]})
            else:
                self.doc(flavor,'assets/Classes/Widget.epokclass',{'version':1,'class':ids['class'],'template':template})
                script=project/'assets/scripts'/('Widget.hpp' if flavor=='cpp' else 'Widget.lua');script.parent.mkdir(parents=True,exist_ok=True);script.write_text('chosen-language rules')

    def doc(self,flavor,name,value):
        path=self.projects[flavor]/name;path.parent.mkdir(parents=True,exist_ok=True);path.write_text(documents.dumps(value),encoding='utf-8')

    def load(self,flavor,name):return documents.loads((self.projects[flavor]/name).read_text(encoding='utf-8'))

    def package(self,flavor,name,payload,asset_id=None,kind=None):
        path=self.projects[flavor]/f'assets/{name}.epokasset'
        if path.is_file():
            data=path.read_bytes();n=int.from_bytes(data[8:12],'little');meta=json.loads(data[16:16+n])
        else:meta={'version':2,'id':asset_id,'kind':kind,'importer_version':1,'source':f'assets/{name}.epokasset','settings':{'type':'Authored'}}
        if not isinstance(payload,bytes):payload=json.dumps(payload).encode()
        meta['source_hash']=hashlib.sha256(payload).hexdigest();metadata=json.dumps(meta).encode()
        path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(b'EPOKAS01'+len(metadata).to_bytes(4,'little')+len(payload).to_bytes(4,'little')+metadata+payload)

    def normalized(self,flavor):return a.shared_resources(self.projects[flavor],documents,self.roles[flavor])[0]

    def different(self,flavor='blueprint'):self.assertNotEqual(self.normalized('cpp'),self.normalized(flavor))

    def test_three_storage_variants_and_renamed_uuids_preserve_compositions(self):
        self.assertEqual(self.normalized('cpp'),self.normalized('blueprint'))
        self.assertEqual(self.normalized('cpp'),self.normalized('lua'))

    def test_pixel_payload_difference_is_retained(self):
        self.package('blueprint','A',b'changed pixels');self.different()

    def test_wrong_existing_asset_reference_is_retained(self):
        scene=self.load('blueprint','assets/scenes/Main.epokmap');scene['actors'][0]['components'][1]['properties']['texture']=self.ids['blueprint']['textureB'];self.doc('blueprint','assets/scenes/Main.epokmap',scene);self.different()

    def test_unknown_uuid_reference_is_retained(self):
        scene=self.load('blueprint','assets/scenes/Main.epokmap');scene['actors'][0]['properties']['child']=str(uuid.uuid4());self.doc('blueprint','assets/scenes/Main.epokmap',scene);self.different()

    def test_wrong_existing_actor_reference_is_retained(self):
        scene=self.load('blueprint','assets/scenes/Main.epokmap');scene['actors'][0]['properties']['child']=self.ids['blueprint']['root'];self.doc('blueprint','assets/scenes/Main.epokmap',scene);self.different()

    def test_blueprint_child_layout_is_retained(self):
        doc=self.load('blueprint','assets/Blueprints/Widget.epokbp');doc['template']['actors'][1]['entity']['components'][1]['properties']['position'][0]=99;self.doc('blueprint','assets/Blueprints/Widget.epokbp',doc);self.different()

    def test_missing_blueprint_composition_is_retained(self):
        doc=self.load('blueprint','assets/Blueprints/Widget.epokbp');doc['template']={'actors':[]};self.doc('blueprint','assets/Blueprints/Widget.epokbp',doc);self.different()

    def test_wrong_composition_binding_is_retained(self):
        doc=self.load('blueprint','assets/Blueprints/Widget.epokbp');doc['template']['references'][0]['target']=self.ids['blueprint']['root'];self.doc('blueprint','assets/Blueprints/Widget.epokbp',doc);self.different()

    def test_wrong_class_uuid_is_not_hidden_by_known_name(self):
        scene=self.load('blueprint','assets/scenes/Main.epokmap');scene['actors'][0]['class']['class_id']=str(uuid.uuid4());self.doc('blueprint','assets/scenes/Main.epokmap',scene);self.different()

    def test_nonstartup_map_changes_are_retained(self):
        self.doc('blueprint','assets/scenes/Other.epokmap',{'actors':[],'ambient':[2,2,3]});self.different()

    def test_authored_tile_collision_changes_are_retained(self):
        self.package('blueprint','Tiles',{'texture':self.ids['blueprint']['textureA'],'tiles':[{'collision':'None','probability':1.0}]});self.different()

    def test_authored_recipe_changes_are_retained_despite_derived_cache_hash(self):
        self.package('blueprint','Recipe',{'steps':[{'step':'Lattice','size':[7,5]}]});self.different()

    def test_scene_lighting_changes_are_retained_despite_derived_bake_hash(self):
        scene=self.load('blueprint','assets/scenes/Main.epokmap');scene['bake']['enabled']=False;self.doc('blueprint','assets/scenes/Main.epokmap',scene);self.different()

    def test_invalid_package_source_hash_is_rejected(self):
        path=self.projects['blueprint']/'assets/A.epokasset';path.write_bytes(path.read_bytes()[:-1]+b'X')
        with self.assertRaisesRegex(RuntimeError,'source hash mismatch'):self.normalized('blueprint')

    def test_invalid_package_length_is_rejected(self):
        path=self.projects['blueprint']/'assets/A.epokasset';path.write_bytes(path.read_bytes()+b'X')
        with self.assertRaisesRegex(RuntimeError,'package length'):self.normalized('blueprint')

    def test_duplicate_asset_uuid_is_rejected(self):
        path=self.projects['blueprint']/'assets/A.epokasset';(path.parent/'Duplicate.epokasset').write_bytes(path.read_bytes())
        with self.assertRaisesRegex(RuntimeError,'Duplicate asset UUID'):self.normalized('blueprint')

    def test_duplicate_provider_composition_is_rejected(self):
        bp=self.load('blueprint','assets/Blueprints/Widget.epokbp');self.doc('blueprint','assets/Classes/Duplicate.epokclass',{'class':bp['id'],'template':bp['template']})
        with self.assertRaisesRegex(RuntimeError,'Duplicate composition'):self.normalized('blueprint')

    def test_unclassified_companion_metadata_is_rejected(self):
        doc=self.load('cpp','assets/Classes/Widget.epokclass');doc['silently_dropped_rule']=17;self.doc('cpp','assets/Classes/Widget.epokclass',doc)
        with self.assertRaisesRegex(RuntimeError,'Unclassified composition metadata'):self.normalized('cpp')


if __name__=='__main__':unittest.main(verbosity=2)
