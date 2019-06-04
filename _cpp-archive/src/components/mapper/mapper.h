#ifndef MAPPER_H
#define MAPPER_H

#include <stdlib.h>

#include "predictor.h"

struct prediction_result {
    bool prediction;
    std::string document;
};

struct prediction_result predict_and_map(const std::string& html, const std::string& url);

// int map_document()

#endif
