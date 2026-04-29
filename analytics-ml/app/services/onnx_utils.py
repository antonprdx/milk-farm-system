import os

import numpy as np


def save_model_onnx(model, features: list[str], path: str, task: str = "classify") -> None:
    import onnx
    import onnxruntime as ort
    from skl2onnx import convert_sklearn
    from skl2onnx.common.data_types import FloatTensorType

    n_features = len(features)
    initial_type = [("float_input", FloatTensorType([None, n_features]))]

    if task == "classify":
        onnx_model = convert_sklearn(
            model, initial_types=initial_type,
            options={id(model): {"zipmap": False}},
        )
    else:
        onnx_model = convert_sklearn(model, initial_types=initial_type)

    meta = onnx_model.metadata_props.add()
    meta.key = "feature_columns"
    meta.value = ",".join(features)
    meta2 = onnx_model.metadata_props.add()
    meta2.key = "task"
    meta2.value = task

    onnx.checker.check_model(onnx_model)
    onnx.save_model(onnx_model, path)


def load_model_onnx(path: str):
    import onnxruntime as ort

    session = ort.InferenceSession(path)
    meta = session.get_modelmeta().custom_metadata_map
    features = meta.get("feature_columns", "").split(",")
    task = meta.get("task", "classify")
    return session, features, task


def predict_onnx(session, X: np.ndarray) -> np.ndarray:
    input_name = session.get_inputs()[0].name
    X_float = X.astype(np.float32)
    outputs = session.run(None, {input_name: X_float})
    if len(outputs) >= 2:
        return outputs[1]
    return outputs[0]
