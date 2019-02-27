#include <stdio.h>
#include <stdlib.h>

// #include <iostream>
// #include <map>
// #include <iterator>
// #include <vector>
// #include <regex>
// #include <cmath>
// #include <iomanip>
// #include <limits>
// #include <algorithm>

#include "predictor.h"

struct res_html load_html_file(const char* filename)
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

    char *html = (char*)malloc(size + 1);
    if(html == NULL) {
        fprintf(stderr, "Can't allocate mem for html file: %s\n", filename);
        exit(EXIT_FAILURE);
    }

    size_t nread = fread(html, 1, size, fh);
    if (nread != size) {
        fprintf(stderr, "could not read %ld bytes ( %ld bytes done)\n", size, nread);
        exit(EXIT_FAILURE);
    }

    fclose(fh);

    struct res_html res = {html, (size_t)size};
    return res;
}


int main(int argc, char* argv[]) {
	try{
		const char* path = argv[1];
        std::string url = argv[2];
		struct res_html res = load_html_file(path);
        int result = predict_html(res.html, res.size, url);
        fprintf(stdout, "Prediction: %d", result);
	} catch (const std::exception& err) {
		std::cout << "Program crashed : " << err.what() << std::endl;
	}

  return 0;
}
