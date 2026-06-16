import json

from grandedge_ml.export import write_fixture_bundle


def test_export_writes_required_bundle_files(tmp_path) -> None:
    bundle = write_fixture_bundle(output_root=tmp_path)

    expected_names = {"model_card", "feature_schema", "calibration", "model"}
    assert expected_names == set(bundle.files)
    for path in bundle.files.values():
        assert path.exists()

    model_card = json.loads(bundle.files["model_card"].read_text(encoding="utf-8"))
    feature_schema = json.loads(bundle.files["feature_schema"].read_text(encoding="utf-8"))
    calibration = json.loads(bundle.files["calibration"].read_text(encoding="utf-8"))

    assert model_card["feature_schema_hash"].startswith("sha256:")
    assert feature_schema["feature_schema_hash"] == model_card["feature_schema_hash"]
    assert calibration["method"] == "identity"
