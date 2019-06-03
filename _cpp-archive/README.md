# Brave SpeedReader

**Private Prototype Repository**

This is the beginning of a prototype SpeedReader implementation that is intended to work across different environments of Brave.

At a high level, SpeedReader:

- Distills text-focused document content from a suitable HTML
- Works on HTML documents before rendering them
- Generates HTML output with no external styling or scripting
- Content styled with Brave-designed themes


## Dependencies

Dependend on [MyHTML parser](https://github.com/lexborisov/myhtml), linked as a submodule, with the required configuration (statically linnked library, no thread support) built together with SpeedReader


## Building

Not thoroughly tested, but a standard C++ toolchain should be sufficient, uses `Make`:

```
make standalone
```

builds a standalone binary that can be used for demo purposes.

Using an included example document this can be used by specifyin the function to apply (`map`) and passing the input HTML filename and the URL string as parameters:

```
./build/speedreader map examples/2CdyGKStt9jwu5u.html $(< examples/2CdyGKStt9jwu5u.url.txt)
```


## Structure

SpeedReader comes in two distinct components: classifier and mapper. The former decides whether reader mode transformation is applicable to the HTML document, and the latter performs the transformation.

The API is currently very simple:

- Classifier alone (defined in `src/components/classifier/predictor.h`) can be invoked with:
```
bool result = predict_html(html, url); // html and url are std::string parameters
```

- Mapper (defined in `src/components/mapper/mapper.h`) can be invoked with:
```
struct prediction_result result = predict_html(html, url); // html and url are std::string
```

In the latter case, mapper calls classifier internally, returning a structure with:

```
struct prediction_result {
	bool prediction;
    std::string document;
}
```

where `document` is an empty string if `prediction = false`.

**Note: currently the mapper only performs an identity transformation, i.e. 1-to-1 mapping from the input, so the document should be exactly as the input if `prediction = true`.**
