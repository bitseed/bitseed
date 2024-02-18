#include <nlohmann/json.hpp>
#include <emscripten.h>
using json = nlohmann::json;

#ifdef __cplusplus
extern "C"{
#endif

uint32_t hash_str_uint32(const std::string& str) {

    uint32_t hash = 0x811c9dc5;
    uint32_t prime = 0x1000193;

    for(int i = 0; i < str.size(); ++i) {
        uint8_t value = str[i];
        hash = hash ^ value;
        hash *= prime;
    }

    return hash;
}

EMSCRIPTEN_KEEPALIVE const char * inscribe_generate(char *seed, char* user_input, const char* attrs) {
    std::vector<char> vec;
    vec.insert(vec.end(), attrs, attrs+strlen(attrs));
    json json_object = json::parse(vec.begin(), vec.end());

    uint32_t hash_value = hash_str_uint32(std::string(seed) + std::string(user_input));

    json json_output;

    if ((!json_object.empty()) && (json_object.is_array())) {
        for (json::iterator it = json_object.begin(); it != json_object.end(); ++it) {
            json attr = *it;
            if (attr.is_object()) {
                for (json::iterator it_inner = attr.begin(); it_inner != attr.end(); ++it_inner) {
                    json attr_value = it_inner.value();
                    if (attr_value.is_object()) {
                        std::string attr_key = it_inner.key();
                        if ((attr_value.contains("data")) && (attr_value.contains("type"))) {
                            std::string attr_type = attr_value["type"];
                            if (attr_type == "range") {
                                json attr_data = attr_value["data"];
                                uint32_t range_min = attr_data["min"];
                                uint32_t range_max = attr_data["max"];
                                uint32_t random_value = range_min + (hash_value % (range_max - range_min + 1));
                                json_output.emplace("id", user_input);
                                json_output.emplace(attr_key, random_value);
                            }
                        }
                    }
                }
            }
        }
    }

    std::string dump = json_output.dump();
    char * output = dump.data();
    char * buffer = (char *)malloc(strlen(output));
    memcpy(buffer, output, strlen(output));
    return buffer;
}

#ifdef __cplusplus
}
#endif
