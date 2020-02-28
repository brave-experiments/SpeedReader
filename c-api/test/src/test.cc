#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "../../include/speedreader.h"
#include "deps/picotest/picotest.h"

using namespace speedreader_ffi;

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

void output_sink_print(const char* chunk, size_t chunk_len) {
  printf("output chunk (len %zu): %.*s\n", chunk_len, (int)chunk_len, chunk);
}

void test_find_type_streaming(C_SpeedReader* speedreader) {
  const char* url_str = "https://cnn.com/news/article/topic/index.html";
  C_CRewriterType type =
      speedreader_find_type(speedreader, url_str, strlen(url_str));
  ok(type == C_CRewriterType::RewriterStreaming);
}

void test_find_type_unknown(C_SpeedReader* speedreader) {
  const char* url_str = "https://example.com/news/article/topic/index.html";
  C_CRewriterType type =
      speedreader_find_type(speedreader, url_str, strlen(url_str));
  ok(type == C_CRewriterType::RewriterUnknown);
}

void test_find_type() {
  C_SpeedReader* speedreader = speedreader_new();
  test_find_type_streaming(speedreader);
  test_find_type_unknown(speedreader);
  speedreader_free(speedreader);
}

void test_rewriter_chunks(C_SpeedReader* speedreader) {
  const char* url_str = "https://cnn.com/news/article/topic/index.html";
  C_CSpeedReaderProcessor* rewriter =
      speedreader_get_rewriter(speedreader, url_str, strlen(url_str),
                               output_sink_print, C_CRewriterType::RewriterUnknown);
  const char* content1 = "<html><div class=\"pg-headline\">";
  ok(speedreader_processor_write(rewriter, content1, strlen(content1)) == 0);
  const char* content2 = "hello world</div></html>";
  ok(speedreader_processor_write(rewriter, content2, strlen(content2)) == 0);
  ok(speedreader_processor_end(rewriter) == 0);
}

void test_rewriter_indirection(C_SpeedReader* speedreader) {
  const char* url_str = "https://cnn.com/news/article/topic/index.html";
  C_CSpeedReaderProcessor* rewriter =
      speedreader_get_rewriter(speedreader, url_str, strlen(url_str),
                               output_sink_print, C_CRewriterType::RewriterUnknown);
  const char* content1 = "<html><div class=\"pg-headline\">";
  ok(speedreader_processor_write(rewriter, content1, strlen(content1)) == 0);
  const char* content2 = "hello world</div></html>";
  ok(speedreader_processor_write(rewriter, content2, strlen(content2)) == 0);
  ok(speedreader_processor_end(rewriter) == 0);
}

void test_rewriter() {
  C_SpeedReader* speedreader = speedreader_new();
  test_rewriter_chunks(speedreader);
  speedreader_free(speedreader);
}

extern "C" {

int run_tests() {
  subtest("url readability", test_url_check);
  subtest("rewriter type", test_find_type);
  subtest("opaque config works", test_find_type);
  subtest("rewriter works", test_rewriter);
  return done_testing();
}

}
