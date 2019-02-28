
#include "predictor.h"

#ifdef SPEEDREADER_FEATURES_DEBUG
#include <iostream>
#endif

float path_counter(std::string s){
	std::regex r("[^/]+");
	std::smatch m;
	std::vector<std::string> paths;
	float path_length;
	for(std::sregex_iterator i = std::sregex_iterator(s.begin(), s.end(), r); i != std::sregex_iterator(); ++i ) {
		std::smatch m = *i;
		paths.push_back(m[0]);
	}
	if (paths.at(0) == "http:" || paths.at(0) == "https:")
		path_length = paths.size() - 2;
	else
		path_length = paths.size() - 1;
	return path_length;
}

std::map <std::string, float> extract_features(const std::string &html, const std::string &url){
    std::map <std::string, float> features;
    try{
        // basic init
        myhtml_t* myhtml = myhtml_create();
        myhtml_init(myhtml, MyHTML_OPTIONS_DEFAULT, 1, 0);

        // init tree
        myhtml_tree_t* tree = myhtml_tree_create();
        myhtml_tree_init(tree, myhtml);

        // parse html
        myhtml_parse(tree, MyENCODING_UTF_8, html.c_str(), html.size());

        #ifdef SPEEDREADER_FEATURES_DEBUG
        std::cout << "MyHTML parsed, extracting features" << std::endl;
        #endif

        features = extract_features_parsed(tree, url, html);

        #ifdef SPEEDREADER_FEATURES_DEBUG
        std::cout << "Features extracted, cleaning up" << std::endl;
        #endif
        
        myhtml_tree_destroy(tree);
        myhtml_destroy(myhtml);
    } catch (const std::exception& err) {
        #ifdef SPEEDREADER_FEATURES_DEBUG
        std::cout << "Program crashed : " << err.what() << std::endl;
        #endif
    }
    return features;
}

std::map <std::string, float> extract_features_parsed(myhtml_tree_t* tree, const std::string &url, const std::string &html){
    std::map <std::string, float> features;
    //images, script, a
    std::string additional_tags[3] = {"img", "a", "script",};
    for(int i=0; i < 3; i++){
        myhtml_collection_t *collection = myhtml_get_nodes_by_name(tree, NULL, additional_tags[i].c_str(), additional_tags[i].size(), NULL);
        if(collection && collection->list && collection->length) {
            if(additional_tags[i] == "img")
                features["images"] = collection->length;
            else if(additional_tags[i] == "a")
                features["anchors"] = collection->length;
            else if(additional_tags[i] == "script")
                features["scripts"] = collection->length;
        } else{
            if(additional_tags[i] == "img")
                features["images"] = 0;
            else if(additional_tags[i] == "a")
                features["anchors"] = 0;
            else if(additional_tags[i] == "script")
                features["scripts"] = 0;
        }

        myhtml_collection_destroy(collection);
    }
    features["text_blocks"] = 0;
    features["words"] = 0;

    #ifdef SPEEDREADER_FEATURES_DEBUG
        std::cout << "Counting tags" << std::endl;
    #endif

    std::string tags[11] = {"blockquote", "dl", "div", "ol", "p", "pre", "table", "ul", "select", "article", "section"};

    for(int i = 0; i < 11; i++){
        myhtml_collection_t *collection = myhtml_get_nodes_by_name(tree, NULL, tags[i].c_str(), tags[i].size(), NULL);
        if(collection && collection->list && collection->length) {
            myhtml_tree_node_t *text_node = myhtml_node_child(collection->list[0]);
            features[tags[i]] = collection->length;
            if(text_node) {
                const char* text = myhtml_node_text(text_node, NULL);
                if(text){
                    std::string t(text);
                    t.erase( std::remove_if( t.begin(), t.end(), ::isspace ), t.end() );
                    if(t.size() >= CHAR_THRESHOLD){
                        features["text_blocks"] ++;
                        features["words"] += std::count(t.begin(), t.end(), ' ')+1;
                    }
                }
            }
        } else {
            features[tags[i]] = 0;
        }
        myhtml_collection_destroy(collection);
    }

    //AMP
    #ifdef SPEEDREADER_FEATURES_DEBUG
        std::cout << "Detecting AMP" << std::endl;
    #endif
    myhtml_collection_t *link_list = myhtml_get_nodes_by_name(tree, NULL, "link", 4, NULL);
    myhtml_collection_t* link_collection = myhtml_get_nodes_by_attribute_value(tree, 
        link_list, NULL, true, "rel", 3, "amphtml", 7, NULL);
    if(link_collection && link_collection->list && link_collection->length) {
        features["amphtml"] = 1;
    } else {
        features["amphtml"] = 0;
    }
    myhtml_collection_destroy(link_collection);
    
    //fb:pages
    #ifdef SPEEDREADER_FEATURES_DEBUG
        std::cout << "Detecting fb_pages" << std::endl;
    #endif
    myhtml_collection_t *meta_list = myhtml_get_nodes_by_name(tree, NULL, "meta", 4, NULL);
    myhtml_collection_t* meta_collection = myhtml_get_nodes_by_attribute_value(tree,
        meta_list, NULL, true, "property", 8, "fb:pages", 8, NULL);
    if(meta_collection && meta_collection->list && meta_collection->length) {
        features["fb_pages"] = 1;
    } else {
        features["fb_pages"] = 0;
    }
    myhtml_collection_destroy(meta_collection);
    

    // FB og:article
    #ifdef SPEEDREADER_FEATURES_DEBUG
        std::cout << "Finding og_article" << std::endl;
    #endif

    myhtml_collection_t* og_type_collection = myhtml_get_nodes_by_attribute_value(tree, NULL, NULL, true, "property", 8, "og:type", 7, NULL);
    features["og_article"] = 0;
    if(og_type_collection) {
        for(size_t i = 0; i < og_type_collection->length; i++) {
            // get attr by key name
            myhtml_tree_attr_t *gets_attr = myhtml_attribute_by_key(og_type_collection->list[i], "content", 7);
            const char *attr_char = myhtml_attribute_value(gets_attr, NULL);
            if (strcmp("article", attr_char) == 0) {
                features["og_article"] = 1;
            }
        }
    }

    // schema.org annotations present

    #ifdef SPEEDREADER_FEATURES_DEBUG
        std::cout << "Finding schema.org annotations with regexes" << std::endl;
    #endif

    std::string schemaCandidates[] = {
        "://schema.org/Article",
        "://schema.org/NewsArticle",
        "://schema.org/APIReference",
    };

    bool schemaPresent = 
        std::search(html.begin(), html.end(), schemaCandidates[0].begin(), schemaCandidates[0].end()) != html.end() ||
        std::search(html.begin(), html.end(), schemaCandidates[1].begin(), schemaCandidates[1].end()) != html.end() ||
        std::search(html.begin(), html.end(), schemaCandidates[2].begin(), schemaCandidates[2].end()) != html.end();
    // std::regex schemaRe ("http(s)?://schema.org/(Article|NewsArticle|APIReference)");

    
    // std::smatch sm;
    // std::regex_search(wat, sm, schemaRe, std::regex_constants::match_any);
    
    if (schemaPresent) {
        features["schema_org_article"] = 1;
    } else {
        features["schema_org_article"] = 0;
    }

    features["url_depth"] = path_counter(url);
    features["file_size"] = html.size();

    return features;    
}

bool predict_features(const std::map<std::string,float> &feature_map) {
    float features[PREDICTOR_FEATURES];
    for (auto i = feature_map.begin(); i != feature_map.end(); i++) {
        
        #ifdef SPEEDREADER_FEATURES_DEBUG
        std::cout << "Feature " << i->first << " = " << i->second << std::endl;
        #endif

        if(i->first == "images") features[0] = i->second;
        else if(i->first == "anchors") features[1] = i->second;
        else if(i->first == "scripts") features[2] = i->second;
        else if(i->first == "text_blocks") features[3] = i->second;
        else if(i->first == "words") features[4] = i->second;
        else if(i->first == "blockquote") features[5] = i->second;
        else if(i->first == "dl") features[6] = i->second;
        else if(i->first == "div") features[7] = i->second;
        else if(i->first == "ol") features[8] = i->second;
        else if(i->first == "p") features[9] = i->second;
        else if(i->first == "pre") features[10] = i->second;
        else if(i->first == "table") features[11] = i->second;
        else if(i->first == "ul") features[12] = i->second;
        else if(i->first == "select") features[13] = i->second;
        else if(i->first == "article") features[14] = i->second;
        else if(i->first == "section") features[15] = i->second;
        else if(i->first == "url_depth")features[16] = i->second;
        else if(i->first == "amphtml") features[17] = i->second;
        else if(i->first == "fb_pages") features[18] = i->second;
        else if(i->first == "og_article") features[19] = i->second;
        else if(i->first == "schema_org_article") features[20] = i->second;
        // else if(i->first == "file_size") features[20] = i->second;
    }

    int prediction = predict(features);
    if (prediction == 1) {
        return true;
    } else {
        return false;
    }
}

bool predict_html(const std::string &html, const std::string &url) {
    std::map<std::string,float> feature_map = extract_features(html, url);
    #ifdef SPEEDREADER_FEATURES_DEBUG
    std::cout << "Features extracted, " << feature_map.size() << " features" << std::endl;
    #endif
    return predict_features(feature_map);
}

