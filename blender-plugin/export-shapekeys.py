from typing import Any
import bpy
import json

bl_info = {
    "name": "Shape Key Exporter",
    "author": "kb10uy",
    "version": (2, 0),
    "blender": (3, 2, 0),
    "location": "",
    "description": "",
    "category": "Object",
}


class ShapeKeyExporter(bpy.types.Operator):
    bl_idname = "kb10uy.shapekeyexporter_export"
    bl_label = "Export Shape Keys"
    bl_description = "Export shape keys as TOML"
    bl_options = {"REGISTER"}

    def execute(self, context: Any) -> set[str]:
        toml_string = ""

        toml_string += "name = \"AvatarName\"\n"
        toml_string += "\n"

        for selected_object in bpy.context.selected_objects:
            if selected_object.type != "MESH":
                self.report(
                    {"WARNING"},
                    f"{selected_object.name} is not a Mesh Object, skipping..."
                )
                continue
            mesh_object: bpy.types.Mesh = selected_object.data
            shape_key_blocks = mesh_object.shape_keys.key_blocks

            toml_string += "[[mesh_groups]]\n"
            toml_string += "mesh = \"Ungrouped\"\n"
            toml_string += f"mesh = \"{selected_object.name}\"\n"

            toml_string += "options = [\n"
            basis_skipped = False
            for name, shape_key in shape_key_blocks.items():
                if not basis_skipped:
                    basis_skipped = True
                    continue
                if len(shape_key.data) == 0:
                    self.report(
                        {"DEBUG"},
                        f"{name} moves no vertex, skipping..."
                    )
                    continue
                toml_string += f"  {{ label = \"{name}\", shapes = [\"{name}\"] }},\n"

            toml_string += "]\n"
            toml_string += "\n"

        self.report({"INFO"}, toml_string)
        return {"FINISHED"}


def register_menu(self: Any, context: Any):
    self.layout.operator(ShapeKeyExporter.bl_idname)


def register():
    bpy.utils.register_class(ShapeKeyExporter)
    bpy.types.VIEW3D_MT_object_context_menu.append(register_menu)


def unregister():
    bpy.utils.unregister_class(ShapeKeyExporter)
    bpy.types.VIEW3D_MT_object_context_menu.remove(register_menu)


if __name__ == "__main__":
    register()
