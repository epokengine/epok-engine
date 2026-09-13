"""Exercise the distribution script in a UUID-isolated HKCU test subtree, not file associations."""
import os
from pathlib import Path
import subprocess
import uuid
import winreg

ROOT=Path(__file__).resolve().parents[2]
EDITOR=ROOT/'target/debug/epok-editor.exe'


def main():
    assert os.name=='nt'
    root='Software\\Epok\\AssociationTests\\'+str(uuid.uuid4())
    extension=root+'\\.epokproject'
    program=root+'\\Epok.Project'
    script=ROOT/'tools/register-project.ps1'
    def run(*extra,ok=True):
        result=subprocess.run(['powershell.exe','-NoProfile','-ExecutionPolicy','Bypass','-File',str(script),'-EditorPath',str(EDITOR),'-TestRegistryRoot','HKCU:\\'+root,*extra],capture_output=True,text=True)
        assert (result.returncode==0)==ok,result.stdout+result.stderr
    def value(path,name=''):
        with winreg.OpenKey(winreg.HKEY_CURRENT_USER,path) as key:return winreg.QueryValueEx(key,name)[0]
    def cleanup(path):
        assert path==root or path.startswith(root+'\\')
        with winreg.OpenKey(winreg.HKEY_CURRENT_USER,path,0,winreg.KEY_READ|winreg.KEY_WRITE) as key:
            children=[]
            for i in range(winreg.QueryInfoKey(key)[0]):children.append(winreg.EnumKey(key,i))
        for child in children:cleanup(path+'\\'+child)
        winreg.DeleteKey(winreg.HKEY_CURRENT_USER,path)
    try:
        with winreg.CreateKey(winreg.HKEY_CURRENT_USER,extension) as key:
            winreg.SetValueEx(key,'',0,winreg.REG_SZ,'Existing.Application')
        run()
        assert value(extension)=='Existing.Application','Existing user defaults must survive registration'
        assert value(program+'\\shell\\open\\command')=='"'+str(EDITOR)+'" "%1"'
        assert value(program+'\\DefaultIcon')=='"'+str(EDITOR)+'",0'
        run() # idempotent reinstallation
        assert value(extension)=='Existing.Application'
        run('-Unregister')
        assert value(extension)=='Existing.Application'
        try:value(program);raise AssertionError('Uninstall retained its ProgID')
        except FileNotFoundError:pass
        with winreg.OpenKey(winreg.HKEY_CURRENT_USER,extension,0,winreg.KEY_WRITE) as key:winreg.DeleteValue(key,'')
        run()
        assert value(extension)=='Epok.Project'
        run('-Unregister')
        try:value(extension);raise AssertionError('Uninstall retained its default association')
        except FileNotFoundError:pass
        print('PASS isolated per-user registration, icon/open quoting, existing-default preservation, reinstall and uninstall')
    finally:
        cleanup(root)


if __name__=='__main__':main()
