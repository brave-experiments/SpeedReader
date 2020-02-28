#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "../../include/speedreader.h"
#include "deps/picotest/picotest.h"

void test_url_readable(C_SpeedReader* speedreader) {
  const char* url_str = "https://cnn.com/news/article/topic/index.html";
  bool readable =
      speedreader_url_readable(speedreader, url_str, strlen(url_str));
  ok(readable == true);
}

void test_url_unreadable(C_SpeedReader* speedreader) {
  const char* url_str = "https://example.com/news/article/topic/index.html";
  bool readable =
      speedreader_url_readable(speedreader, url_str, strlen(url_str));
  ok(readable == false);
}

void test_url_invalid(C_SpeedReader* speedreader) {
  const char* url_str = "brave://about";
  bool readable =
      speedreader_url_readable(speedreader, url_str, strlen(url_str));
  ok(readable == false);
}

void test_url_empty(C_SpeedReader* speedreader) {
  const char* url_str = "";
  bool readable =
      speedreader_url_readable(speedreader, url_str, strlen(url_str));
  ok(readable == false);
}

void test_url_check() {
  C_SpeedReader* speedreader = speedreader_new();
  test_url_readable(speedreader);
  test_url_unreadable(speedreader);
  test_url_invalid(speedreader);
  test_url_empty(speedreader);
  speedreader_free(speedreader);
}

#define UNUSED (void)

void output_sink_print(const char *chunk, size_t chunk_len) {
    printf("output chunk (len %zu): %.*s\n", chunk_len, (int)chunk_len, chunk);
}

void test_find_config() {
  C_SpeedReader* speedreader = speedreader_new();
  const char* url_str = "https://cnn.com/news/article/topic/index.html";

  C_CRewriterType type =
      speedreader_find_type(speedreader, url_str, strlen(url_str));
  ok(type == RewriterStreaming);

  void* config_extras =
      speedreader_find_config_extras(speedreader, url_str, strlen(url_str));

  void* rewriter = speedreader_get_rewriter(
      speedreader, url_str, strlen(url_str),
      type, config_extras,
      output_sink_print
      );
  const char* content =
      "<html><div class=\"pg-headline\">hello world</div></html>";
  int ret = speedreader_processor_write(rewriter, content, strlen(content));
  ok(ret == 0);
  // ret = speedreader_processor_end(rewriter);
  // ok(ret == 0);

  // speedreader_processor_free(rewriter, config_extras);
  speedreader_free(speedreader);
}

int run_tests() {
  subtest("url check", test_url_check);
  subtest("find config check", test_find_config);
  return done_testing();
}
