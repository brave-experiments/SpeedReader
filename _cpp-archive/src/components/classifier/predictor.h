#ifndef PREDICTOR_H
#define PREDICTOR_H

#include <stdlib.h>
#include <map>
#include <regex>

#include <myhtml/api.h>

#include "model.h"

const int CHAR_THRESHOLD = 400;
const int PREDICTOR_FEATURES = 21;

struct res_html {
    char  *html;
    size_t size;
};

std::map <std::string, float> extract_features(const std::string &html, const std::string &url);
std::map <std::string, float> extract_features_parsed(myhtml_tree_t* tree, const std::string &url, const std::string &html);

bool predict_html(const std::string &html, const std::string &url);
bool predict_features(const std::map<std::string,float> &feature_map);

// #define SPEEDREADER_FEATURES_DEBUG

#endif