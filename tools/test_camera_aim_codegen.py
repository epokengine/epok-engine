"""Check the portable integer aim kernel with the actual PSX cross compiler.

No cycle/FPS claim: only code generation and absence of out-of-line helpers.
"""
from pathlib import Path
import re
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
BIN = ROOT/'.tools/mips/bin'
SUFFIX = '.exe' if (BIN/'mipsel-none-elf-g++.exe').exists() else ''

class CameraAimCodegen(unittest.TestCase):
    def test_direction_and_controller_cross_compile(self):
        with tempfile.TemporaryDirectory(prefix='epok-rail-aim-') as folder:
            for optimization in ('-O2','-O3'):
                with self.subTest(optimization=optimization):
                    obj=Path(folder)/'rail.o'
                    command=[str(BIN/f'mipsel-none-elf-g++{SUFFIX}'),'-std=c++20',optimization,
                             '-march=r3000','-mabi=32','-ffreestanding','-fno-exceptions','-fno-rtti',
                             '-Wall','-Wextra','-Werror','-I',str(ROOT/'runtime'),'-c',
                             str(ROOT/'tests/runtime/rail_camera_aim_codegen.cpp'),'-o',str(obj)]
                    result=subprocess.run(command,capture_output=True,text=True)
                    self.assertEqual(result.returncode,0,result.stdout+result.stderr)
                    undefined=subprocess.check_output([str(BIN/f'mipsel-none-elf-nm{SUFFIX}'),'-u',str(obj)],text=True)
                    self.assertEqual(undefined.strip(),'')

    def test_standalone_integer_kernel(self):
        with tempfile.TemporaryDirectory(prefix='epok-camera-aim-') as folder:
            for optimization in ('-O2','-O3'):
                with self.subTest(optimization=optimization):
                    obj = Path(folder)/'aim.o'
                    command = [str(BIN/f'mipsel-none-elf-g++{SUFFIX}'),'-std=c++20',optimization,
                               '-march=r3000','-mabi=32','-ffreestanding','-fno-exceptions','-fno-rtti',
                               '-Wall','-Wextra','-Werror','-I',str(ROOT/'runtime'),'-c',
                               str(ROOT/'tests/runtime/camera_aim_codegen.cpp'),'-o',str(obj)]
                    result = subprocess.run(command,capture_output=True,text=True)
                    self.assertEqual(result.returncode,0,result.stdout+result.stderr)
                    undefined = subprocess.check_output([str(BIN/f'mipsel-none-elf-nm{SUFFIX}'),'-u',str(obj)],text=True)
                    self.assertEqual(undefined.strip(),'')
                    code = subprocess.check_output([str(BIN/f'mipsel-none-elf-objdump{SUFFIX}'),'-d',str(obj)],text=True)
                    self.assertIn('<camera_aim>:',code)
                    self.assertFalse(re.search(r'\b(?:jal|jalr|div|divu|lwc1|swc1|cvt\.\w+)\b',code),code)
                    self.assertIn('sra',code)  # Signed floor for negative products.

if __name__ == '__main__': unittest.main()
