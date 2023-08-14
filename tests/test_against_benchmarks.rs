//! This file was generated by tests/generate_test_cases.sh.

mod common;

test_against_benchmark!(test_efficiency, recursiveSpheres3);
test_against_benchmark!(test_efficiency, recursiveSpheres2);
test_against_benchmark!(test_efficiency, recursiveCones4);
test_against_benchmark!(test_efficiency, recursiveSpheres1);
test_against_benchmark!(test_efficiency, recursiveCubes4);
test_against_benchmark!(test_efficiency, recursiveSpheres4);
test_against_benchmark!(test_intersect, behind_camera_sphere);
test_against_benchmark!(test_intersect, phong_total);
test_against_benchmark!(test_intersect, parse_matrix);
test_against_benchmark!(test_intersect, phong_ambient);
test_against_benchmark!(test_intersect, phong_diffuse);
test_against_benchmark!(test_intersect, behind_camera);
test_against_benchmark!(test_intersect, phong_specular);
test_against_benchmark!(test_light, directional_light_2);
test_against_benchmark!(test_light, directional_light_1);
test_against_benchmark!(test_light, spot_light_1);
test_against_benchmark!(test_light, point_light_2);
test_against_benchmark!(test_light, spot_light_2);
test_against_benchmark!(test_light, point_light_1);
test_against_benchmark!(test_unit, unit_cylinder);
test_against_benchmark!(test_unit, unit_cube);
test_against_benchmark!(test_unit, unit_sphere);
test_against_benchmark!(test_unit, unit_cone);
test_against_benchmark!(test_feature, texture_cone);
test_against_benchmark!(test_feature, texture_sphere);
test_against_benchmark!(test_feature, mirror_refl);
test_against_benchmark!(test_feature, texture_cube);
test_against_benchmark!(test_feature, mirror_depth);
test_against_benchmark!(test_feature, texture_cone2);
test_against_benchmark!(test_feature, reflection);
test_against_benchmark!(test_feature, texture_uv);
test_against_benchmark!(test_feature, texture_cyl);
test_against_benchmark!(test_feature, attenuation);
test_against_benchmark!(test_feature, shadow_test);
test_against_benchmark!(test_feature, texture_cyl2);
test_against_benchmark!(test_feature, shadow_special_case);
test_against_benchmark!(test_feature, texture_cheese);
