from typing import Any
import bpy
import json

bl_info = {
    "name": "Shape Key Exporter",
    "author": "kb10uy",
    "version": (0, 5),
    "blender": (3, 2, 0),
    "location": "",
    "description": "",
    "category": "Object",
}


class ShapeKeyExporter(bpy.types.Operator):
    bl_idname = "kb10uy.shapekeyexporter_export"
    bl_label = "Export Shape Keys"
    bl_description = "Export shape keys as JSON"
    bl_options = {"REGISTER"}

    def execute(self, context: Any) -> set[str]:
        shape_key_table = {}
        for selected_object in bpy.context.selected_objects:
            if selected_object.type != "MESH":
                self.report(
                    {"WARNING"},
                    f"{selected_object.name} is not a Mesh Object, skipping..."
                )
                continue
            mesh_object: bpy.types.Mesh = selected_object.data
            shape_key_blocks = mesh_object.shape_keys.key_blocks

            object_shape_keys = []
            basis_skipped = False
            parameter_index = 1
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

                object_shape_keys.append({
                    "animation_name": name,
                    "shape_key_name": name,
                    "index": parameter_index,
                })
                parameter_index += 1

            self.report(
                {"DEBUG"},
                f"Exported {len(object_shape_keys)} key(s) for {mesh_object.name}"
            )
            shape_key_table[selected_object.name] = {
                "Uncategorized": {
                    "emit": False,
                    "type": "select",
                    "keys": object_shape_keys,
                },
            }

        shape_key_table_json = json.dumps(
            {
                "animation_path": "Assets/",
                "exported_objects": shape_key_table,
            },
            ensure_ascii=False,
            indent=4
        )

        self.report({"INFO"}, shape_key_table_json)
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
