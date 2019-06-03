#include "mapper.h"

#include <myhtml/api.h>

std::string map_document(myhtml_tree_t* tree);

struct prediction_result predict_and_map(const std::string& html, const std::string& url) {
    // basic init
    myhtml_t* myhtml = myhtml_create();
    myhtml_init(myhtml, MyHTML_OPTIONS_DEFAULT, 1, 0);

    // init tree
    myhtml_tree_t* tree = myhtml_tree_create();
    myhtml_tree_init(tree, myhtml);

    // parse html
    myhtml_parse(tree, MyENCODING_UTF_8, html.c_str(), html.size());

    bool prediction = predict_features(extract_features_parsed(tree, url, html));

    // Re-serialized, mapped document. Null if no transformation performed
    std::string mapped;

    if (prediction) {
        mapped = map_document(tree);
    } else {
        // false prediction - no mapping
    }

    // cleanup the allocated structures
    myhtml_tree_destroy(tree);
    myhtml_destroy(myhtml);

    // return the prediction
    struct prediction_result res = {prediction, mapped};
    return res;
}

std::string map_document(myhtml_tree_t* tree) {
    mycore_string_raw_t str = {0};
    myhtml_serialization_tree_buffer(myhtml_tree_get_document(tree), &str);
    // convert MyHTML structure to std::string
    std::string mapped(str.data);

    return mapped;
}