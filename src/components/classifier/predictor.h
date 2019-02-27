#ifndef PREDICTOR_H
#define PREDICTOR_H

#include <stdio.h>
#include <stdlib.h>
#include <iostream>
#include <map>
#include <iterator>
#include <vector>
#include <regex>
#include <cmath>
#include <iomanip>
#include <limits>
#include <algorithm>

#include <myhtml/api.h>

#include "model.h"

const int CHAR_THRESHOLD = 400;
const int PREDICTOR_FEATURES = 21;

struct res_html {
    char  *html;
    size_t size;
};

std::map <std::string, float> extract_features(char *html, size_t size, std::string url);

int predict_html(char *html, size_t size, std::string url);

#endif