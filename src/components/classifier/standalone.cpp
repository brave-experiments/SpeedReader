#include <stdio.h>
#include <stdlib.h>
#include <iostream>

#include "predictor.h"
#include "mapper.h"

// #define SPEEDREADER_DEBUG

std::string load_html_file(const char* filename)
{
    FILE *fh = fopen(filename, "rb");
    if(fh == NULL) {
        fprintf(stderr, "Can't open html file: %s\n", filename);
        exit(EXIT_FAILURE);
    }

    if(fseek(fh, 0L, SEEK_END) != 0) {
        fprintf(stderr, "Can't set position (fseek) in file: %s\n", filename);
        exit(EXIT_FAILURE);
    }

    long size = ftell(fh);

    if(fseek(fh, 0L, SEEK_SET) != 0) {
        fprintf(stderr, "Can't set position (fseek) in file: %s\n", filename);
        exit(EXIT_FAILURE);
    }

    if(size <= 0) {
        fprintf(stderr, "Can't get file size or file is empty: %s\n", filename);
        exit(EXIT_FAILURE);
    }

    #ifdef SPEEDREADER_DEBUG
    std::cout << "Reading file with size " << size << std::endl;
    #endif

    std::string html(size, '\0');

    size_t nread = fread(&html[0], sizeof(char), (size_t)size, fh);
    if (nread != size) {
        fprintf(stderr, "could not read %ld bytes ( %ld bytes done)\n", size, nread);
        exit(EXIT_FAILURE);
    }

    fclose(fh);

    #ifdef SPEEDREADER_DEBUG
    std::cout << "Returning read HTML" << std::endl;
    #endif
    return html;
}


int main(int argc, char* argv[]) {
    try{
        std::string command(argv[1]);
        const char* path = argv[2];
        std::string url(argv[3]);
        if (command == "predict") {
            std::string html = load_html_file(path);
            bool result = predict_html(html, url);
            std::cout << "Prediction: " << result << std::endl;
        } else if (command == "map") {
            std::string html = load_html_file(path);
            struct prediction_result result = predict_and_map(html, url);
            std::cout << "Prediction: " << result.prediction << std::endl;
            if (result.prediction) {
                std::cout << result.document << std::endl;
            }
        }
    } catch (const std::exception& err) {
        std::cout << "Program crashed : " << err.what() << std::endl;
    }

  return 0;
}
