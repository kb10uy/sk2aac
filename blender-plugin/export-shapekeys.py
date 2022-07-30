from typing import Any
import bpy
import json

bl_info = {
    "name": "Shape Key Exporter",
    "author": "kb10uy",
    "version": (1, 0),
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
        animation_objects = []
        for selected_object in bpy.context.selected_objects:
            if selected_object.type != "MESH":
                self.report(
                    {"WARNING"},
                    f"{selected_object.name} is not a Mesh Object, skipping..."
                )
                continue
            mesh_object: bpy.types.Mesh = selected_object.data
            shape_key_blocks = mesh_object.shape_keys.key_blocks

            animation_object = {
                "name": selected_object.name,
                "groups": [],
            }

            shapes = []
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

                shapes.append({
                    "animation_name": name,
                    "shape_name": name,
                    "index": parameter_index,
                })
                parameter_index += 1

            unset_group = {
                "group_name": "Unset",
                "animation_type": "select",
                "emit": False,
                "shapes": shapes,
            }
            animation_object["groups"] = [unset_group]
            animation_objects.append(animation_object)

            self.report(
                {"DEBUG"},
                f"Exported {len(shapes)} key(s) for {mesh_object.name}"
            )

        animation_descriptor = {
            "animation_path": "Assets/",
            "animation_objects": animation_objects,
        }

        json_string = json.dumps(
            animation_descriptor,
            ensure_ascii=False,
            indent=4
        )

        self.report({"INFO"}, json_string)
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
