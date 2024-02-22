#include <nlohmann/json.hpp>
#include <emscripten.h>
using json = nlohmann::json;

#ifdef __cplusplus
extern "C"{
#endif

EMSCRIPTEN_KEEPALIVE char * test_alloc() {
    char * block_hash_buffer = (char *)malloc(123);
    return block_hash_buffer;
}

struct BlockInfo {
    uint32_t BlockNumber;
    char *BlockHash;
    char *TransactionHash;
};

EMSCRIPTEN_KEEPALIVE BlockInfo * GlobalBlockInfo;

EMSCRIPTEN_KEEPALIVE void initial(uint32_t block_number, char* block_hash, char *transaction_hash) {
    BlockInfo * block_info = (BlockInfo *)malloc(sizeof(BlockInfo));
    block_info->BlockNumber = block_number;
    GlobalBlockInfo = block_info;

    char * block_hash_buffer = (char *)malloc(sizeof(char) * strlen(block_hash));
    memcpy(block_hash_buffer, block_hash, sizeof(char) * strlen(block_hash));
    block_info->BlockHash = block_hash_buffer;

    char * tx_hash_buffer = (char *)malloc(sizeof(char) * strlen(transaction_hash));
    memcpy(tx_hash_buffer, transaction_hash, sizeof(char) * strlen(transaction_hash));
    block_info->TransactionHash = tx_hash_buffer;
}

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

EMSCRIPTEN_KEEPALIVE const char * inscribe_generate(char* user_input, const char* attrs) {
    uint32_t hash_value = hash_str_uint32(std::string(GlobalBlockInfo->BlockHash) +
            std::string(GlobalBlockInfo->TransactionHash) + std::string(user_input));

    std::vector<char> vec;
    vec.insert(vec.end(), attrs, attrs+strlen(attrs));
    json json_object = json::from_cbor(vec.begin(), vec.end());

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

    std::vector<std::uint8_t> dump = json::to_cbor(json_output);
    char * output = (char *)dump.data();
    char * buffer = (char *)malloc(sizeof(char) * strlen(output));
    memcpy(buffer, output, strlen(output));
    return buffer;
}

EMSCRIPTEN_KEEPALIVE bool inscribe_verify(char* user_input, const char* attrs, const char* output) {
    const char *inscribe_output = inscribe_generate(user_input, attrs);
    if (strcmp(inscribe_output, output)) {
        return true;
    } else {
        return false;
    }
}

EMSCRIPTEN_KEEPALIVE const char * indexer_generate(char *inscription_id, const char* attrs) {
    return inscribe_generate(inscription_id, attrs);
}

#ifdef __cplusplus
}
#endif
