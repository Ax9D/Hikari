import os
import subprocess as sp
import shutil
import argparse, sys
from os import path, getcwd, makedirs

CWD = getcwd()
TARGET_PATH = path.join(CWD, "target", "dist")
DIST_PATH = path.join(CWD, "dist")
DIST_TMP_PATH = path.join(CWD, ".dist")

def build_editor():
    return sp.run(["cargo", "build", "--profile=dist", "-p", "hikari_editor"])

def build_cli():
    return sp.run(["cargo", "build", "--profile=dist", "-p", "hikari_cli"])

def copy_folder_structure():
    folders = ["templates", "tools", "engine_assets/shaders", "engine_assets/fonts", "engine_assets/textures"]

    for folder in folders:
        shutil.copytree(path.join(CWD, folder), path.join(DIST_TMP_PATH, folder))

def copy_binaries():
    exes = ["hikari_editor", "hikari_cli"]

    if sys.platform == "win32":
        exes = [exe + ".exe" for exe in exes]
    
    for exe in exes:
        shutil.copy(path.join(TARGET_PATH, exe), path.join(DIST_TMP_PATH, exe))
def copy_files():
    files = ["HIKARI_VERSION", "imgui.ini"]

    for f in files:
        shutil.copyfile(path.join(CWD, f), path.join(DIST_TMP_PATH, f))
    
def eprint(*args, **kwargs):
    print(*args, file=sys.stderr, **kwargs)

def exit():
    shutil.rmtree(DIST_TMP_PATH)
    sys.exit(-1)
def make_archive(source, destination):
    base = path.basename(destination)
    name = base.split('.')[0]
    format = base.split('.')[1]
    shutil.make_archive(name, format, source)

def run(package_path = None, clean = False):
    if path.exists(DIST_TMP_PATH):
        shutil.rmtree(DIST_TMP_PATH)

    makedirs(DIST_TMP_PATH)
    if build_editor().returncode != 0:
        eprint("Failed to build editor")
        exit()
    if build_cli().returncode != 0:
        eprint("Failed to build cli")
        exit()

    copy_folder_structure()
    copy_files()
    copy_binaries()

    if path.exists(DIST_PATH):
        shutil.rmtree(DIST_PATH)
    
    if package_path:
        make_archive(DIST_TMP_PATH, package_path)
    else:
        shutil.move(DIST_TMP_PATH, DIST_PATH)

    if clean:
        sp.run(["cargo", "clean", "--profile=dist"])


parser = argparse.ArgumentParser()

parser.add_argument("--package", type=str, help = "Put files into a compressed archive")
parser.add_argument("--clean", help = "Clean previous build")

args = parser.parse_args()

if args.package:
    (filename, ext) = path.splitext(args.package)
    ext = ext[1:]
    print(ext)
    if ext != "zip":
        eprint("Unsupported archive format, Supported formats are: zip")
        sys.exit(-1)
    
run(args.package, args.clean)
