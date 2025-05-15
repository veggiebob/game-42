"""
Copy and paste this script into the script editor in Blender and run it once to automatically
export the blend file as a gltf in assets/gltf

It will export to the path `assets/gltf/<name>/<name>.glb`

This assumes that you are editing a .blend file in THIS directory, which is in `assets/blender`

"""

import bpy
import os
import pathlib

EXPORT_DIR = "gltf"
ASSET_DIR = pathlib.Path(__file__).parent.parent.parent

CURRENT_FILENAME = pathlib.Path(__file__).parent.name # 'car.blend'
CURRENT_FN_NO_EXT = CURRENT_FILENAME[:CURRENT_FILENAME.rfind('.')] # 'car'
EXPORT_DIR = os.path.join(ASSET_DIR, EXPORT_DIR, CURRENT_FN_NO_EXT) # 'assets/gltf/car/'
EXPORT_FILENAME = CURRENT_FN_NO_EXT + ".glb"

def auto_export(scene):
    filepath = os.path.join(EXPORT_DIR, EXPORT_FILENAME)
    bpy.ops.export_scene.gltf(filepath=filepath, export_apply=True, use_selection=False)

def register():
    bpy.app.handlers.save_post.append(auto_export)

def unregister():
    bpy.app.handlers.save_post.remove(auto_export)

register()