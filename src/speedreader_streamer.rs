use lol_html::doc_comments;
use lol_html::html_content::ContentType;
use lol_html::html_content::Element;
use lol_html::html_content::TextChunk;
use lol_html::html_content::UserData;
use lol_html::ElementContentHandlers;
use lol_html::Selector;
use lol_html::{HtmlRewriter, Settings};
use std::collections::HashMap;
use std::error::Error;

pub type HandlerResult = Result<(), Box<dyn Error>>;
pub type ElementHandler = Box<dyn Fn(&mut Element) -> HandlerResult>;
pub type TextHandler = Box<dyn Fn(&mut TextChunk) -> HandlerResult>;

pub struct ContentFunction {
    pub element: Option<ElementHandler>,
    pub text: Option<TextHandler>,
}

impl From<ElementHandler> for ContentFunction {
    #[inline]
    fn from(handler: ElementHandler) -> Self {
        ContentFunction {
            element: Some(handler),
            text: None,
        }
    }
}

impl From<TextHandler> for ContentFunction {
    #[inline]
    fn from(handler: TextHandler) -> Self {
        ContentFunction {
            element: None,
            text: Some(handler),
        }
    }
}

#[inline]
fn get_content_handlers<'h>(function: &'h ContentFunction) -> ElementContentHandlers<'h> {
    if function.element.is_some() {
        ElementContentHandlers::default().element(function.element.as_ref().unwrap())
    } else if function.text.is_some() {
        ElementContentHandlers::default().text(function.text.as_ref().unwrap())
    } else {
        ElementContentHandlers::default()
    }
}

#[derive(Clone, Debug)]
pub struct AttributeRewrite {
    pub selector: String,
    pub attribute: String,
    pub to_attribute: String,
    pub element_name: String,
}

#[derive(Clone, Debug)]
pub struct SiteConfiguration<'a> {
    pub domain: String,
    pub main_content: Vec<&'a str>,
    pub main_content_cleanup: Vec<&'a str>,
    pub delazify: bool,
    pub fix_embeds: bool,
    pub content_script: Option<String>,
    pub preprocess: Vec<AttributeRewrite>,
}

pub struct SpeedReader {
    element_content_handlers: Vec<(Selector, ContentFunction)>,
}

impl SpeedReader {
    pub fn configure(conf: &SiteConfiguration, origin: &str) -> Self {
        let mut sr = SpeedReader {
            element_content_handlers: vec![],
        };

        for attr_rewrite in &conf.preprocess {
            let rewrite = attr_rewrite.clone();
            sr.add_element_function(
                &attr_rewrite.selector,
                Box::new(move |el| {
                    el.get_attribute(&rewrite.attribute).map(|attr_value| {
                        el.set_attribute(&rewrite.to_attribute, &attr_value)
                            .unwrap_or(());
                    });
                    el.set_tag_name(&rewrite.element_name)?;
                    Ok(())
                }),
            );
        }

        sr.collect_main_content(&conf.main_content, &conf.main_content_cleanup);
        if conf.delazify {
            sr.delazify();
        }
        if conf.fix_embeds {
            sr.fix_social_embeds();
        }
        sr.correct_relative_links(origin.to_owned());

        let maybe_script = conf.content_script.clone();
        maybe_script.map(|script| {
            sr.add_element_function(
                "body",
                Box::new(move |el| {
                    el.append(&script, ContentType::Html);
                    Ok(())
                }),
            )
        });
        sr
    }

    pub fn rewrite(
        &self,
        data: &[u8],
        output: &mut Vec<u8>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut rewriter = HtmlRewriter::try_new(
            Settings {
                element_content_handlers: self
                    .element_content_handlers
                    .iter()
                    .map(|(selector, function)| (selector, get_content_handlers(function)))
                    .collect(),
                document_content_handlers: vec![doc_comments!(|el| Ok(el.remove()))],
                ..Settings::default()
            },
            |c: &[u8]| output.extend_from_slice(c),
        )?;
        rewriter.write(data)?;
        rewriter.end()?;
        Ok(())
    }

    fn add_element_function(&mut self, selector: &str, handler: ElementHandler) {
        self.element_content_handlers.push((
            selector.parse::<Selector>().unwrap(),
            ContentFunction::from(handler),
        ))
    }

    fn add_text_function(&mut self, selector: &str, handler: TextHandler) {
        self.element_content_handlers.push((
            selector.parse::<Selector>().unwrap(),
            ContentFunction::from(handler),
        ))
    }

    fn collect_main_content(&mut self, content_selectors: &[&str], cleanup_selectors: &[&str]) {
        content_selectors.iter().for_each(|selector| {
            self.add_element_function(
                &format!("{}", selector),
                Box::new(SpeedReader::mark_retained_element),
            );
            self.add_text_function(
                &format!("{}", selector),
                Box::new(SpeedReader::mark_retained_text),
            );
            self.add_element_function(
                &format!("{} *", selector),
                Box::new(SpeedReader::mark_retained_element),
            );
            self.add_text_function(
                &format!("{} *", selector),
                Box::new(SpeedReader::mark_retained_text),
            );
        });

        cleanup_selectors.iter().for_each(|selector| {
            self.add_element_function(selector, Box::new(|el| Ok(el.remove())));
        });

        // Drop everything else
        self.add_text_function("*", Box::new(SpeedReader::remove_unmarked_text));
        self.add_element_function("*", Box::new(SpeedReader::unwrap_unmarked_element));
        self.add_element_function("[style]", Box::new(|el| Ok(el.remove_attribute("style"))));
    }

    fn correct_relative_links(&mut self, origin: String) {
        let href_origin = origin.clone();
        self.add_element_function(
            "a[href]",
            Box::new(move |el| {
                let href = el.get_attribute("href").expect("href was required");

                if !href.starts_with("http") {
                    el.set_attribute("href", &format!("{}{}", href_origin, href))?;
                }

                Ok(())
            }),
        );
        let src_origin = origin.clone();
        self.add_element_function(
            "img[src]",
            Box::new(move |el| {
                let src = el.get_attribute("src").expect("src was required");

                if !src.starts_with("http") {
                    el.set_attribute("src", &format!("{}{}", src_origin, src))?;
                }

                Ok(())
            }),
        );
    }

    fn delazify(&mut self) {
        self.add_element_function(
            "[data-src]",
            Box::new(|el| {
                el.get_attribute("data-src").map(|src| {
                    el.set_attribute("src", &src).ok();
                });
                Ok(())
            }),
        );
        self.add_element_function(
            "[data-srcset]",
            Box::new(|el| {
                el.get_attribute("data-srcset").map(|srcset| {
                    el.set_attribute("srcset", &srcset).ok();
                });
                Ok(())
            }),
        );
        self.add_element_function(
            "[data-original]",
            Box::new(|el| {
                el.get_attribute("data-original").map(|original| {
                    el.set_attribute("src", &original).ok();
                });
                Ok(())
            }),
        );
        self.add_element_function(
            "img[data-src-medium]",
            Box::new(|el| {
                el.get_attribute("data-src-medium").map(|original| {
                    el.set_attribute("src", &original).ok();
                });
                Ok(())
            }),
        );
        self.add_element_function(
            "img[data-raw-src]",
            Box::new(|el| {
                el.get_attribute("data-raw-src").map(|original| {
                    el.set_attribute("src", &original).ok();
                });
                Ok(())
            }),
        );
        self.add_element_function(
            "img[data-gl-src]",
            Box::new(|el| {
                el.get_attribute("data-gl-src").map(|original| {
                    el.set_attribute("src", &original).ok();
                });
                Ok(())
            }),
        );
        self.add_element_function(
            "img",
            Box::new(|el| {
                el.remove_attribute("height");
                el.remove_attribute("width");
                Ok(())
            }),
        );
    }

    fn fix_social_embeds(&mut self) {
        self.add_element_function(".twitterContainer", Box::new(|el: &mut Element| {
            el.prepend(r#"<script type="text/javascript" src="//platform.twitter.com/widgets.js" async=""></script>"#, ContentType::Html);
            Ok(())
        }))
    }

    fn mark_retained_element(el: &mut Element) -> HandlerResult {
        Ok(el.set_user_data(true))
    }
    fn mark_retained_text(t: &mut TextChunk) -> HandlerResult {
        Ok(t.set_user_data(true))
    }
    fn remove_unmarked_text(t: &mut TextChunk) -> HandlerResult {
        let user_data = t.user_data_mut().downcast_ref::<bool>();
        if user_data != Some(&true) {
            Ok(t.remove())
        } else {
            Ok(())
        }
    }
    fn unwrap_unmarked_element(el: &mut Element) -> HandlerResult {
        let user_data = el.user_data_mut().downcast_ref::<bool>();
        if user_data != Some(&true) {
            Ok(el.remove_and_keep_content())
        } else {
            Ok(())
        }
    }

    pub fn known_configs() {
        let mut configs = HashMap::new();
        configs.insert(
            "cnet.com",
            SiteConfiguration {
                domain: "cnet.com".to_owned(),
                main_content: vec![".article-main-body", ".hero-content"],
                main_content_cleanup: vec![
                    "footer",
                    "noscript",
                    ".c-head_bottomWrapper",
                    ".c-head_share",
                    ".social-button-small-author",
                    ".clickToEnlarge",
                    ".gallery",
                    ".video",
                    ".svg-symbol",
                ],
                delazify: true,
                fix_embeds: true,
                content_script: None,
                preprocess: vec![],
            },
        );
        configs.insert(
            "247sports.com",
            SiteConfiguration {
                domain: "247sports.com".to_owned(),
                main_content: vec!["section .article-cnt"],
                main_content_cleanup: vec![".article-cnt__header > .container"],
                delazify: true,
                fix_embeds: true,
                content_script: None,
                preprocess: vec![],
            },
        );
        configs.insert("abcnews.go.com",
            SiteConfiguration {
                domain: "abcnews.go.com".to_owned(),
                main_content: vec![".Article__Wrapper", "body > script:not([src])"],
                main_content_cleanup: vec![
                    ".CalloutLink", ".Article__Footer", ".Article__Header .Share",
                    ".MediaPlaceholder__Overlay",
                    ".inlineElement > iframe",
                    ".Screen__Reader__Text", ".taboola"
                ],
                delazify: true,
                fix_embeds: true,
                content_script: Some(r#"<script>
                document.querySelector(".FeaturedMedia figure img").src = 
                    JSON.parse(document.querySelector('script[type="application/ld+json"]').innerText).image.url;
                [...document.querySelectorAll(".InlineImage .Image__Wrapper img")]
                    .map((e, i) => e.src = 
                        __abcnews__.page.content.story.everscroll[0].inlines.filter(d => d.type === "image").map(i => i.imageSrc)[i])
                </script>"#.to_owned()),
                preprocess: vec![],
            }
        );

        configs.insert(
            "cnn.com",
            SiteConfiguration {
                domain: "cnn.com".to_owned(),
                main_content: vec![
                    ".pg-headline",
                    ".metadata",
                    ".media__video--thumbnail-wrapper img",
                    "[itemprop=\"articleBody\"]",
                ],
                main_content_cleanup: vec![
                    ".m-share",
                    ".pg-comments",
                    "[class*=\"outbrain\"]",
                    ".zn-story-bottom",
                    ".zn-body__read-more",
                ],
                delazify: true,
                fix_embeds: true,
                content_script: None,
                preprocess: vec![],
            },
        );

        configs.insert(
            "nytimes.com",
            SiteConfiguration {
                domain: "nytimes.com".to_owned(),
                main_content: vec![
                    "div.g-blocks",
                    "section[name=\"articleBody\"]",
                    "article header",
                ],
                main_content_cleanup: vec![
                    ".ad",
                    "header#story-header",
                    ".story-body-1 .lede.video",
                    ".visually-hidden",
                    "#newsletter-promo",
                    ".promo",
                    ".comments-button",
                    ".hidden",
                    ".comments",
                    ".supplemental",
                    ".nocontent",
                    ".story-footer-links",
                    "#sponsor-wrapper",
                    "[role=\"toolbar\"]",
                    "header > section",
                ],
                delazify: true,
                fix_embeds: true,
                content_script: Some(
                    r#"
            <script>
            [...document.querySelectorAll("figure[itemid]")].forEach(fig => {
                let lazy = fig.querySelector("[data-testid=\"lazyimage-container\"]");
                if (lazy) { lazy.innerHTML = "<img src='" + fig.getAttribute("itemid") + "'>" }
            });
            </script>
            "#
                    .to_owned(),
                ),
                preprocess: vec![],
            },
        );

        configs.insert("theguardian.com",
        SiteConfiguration {
            domain: "theguardian.com".to_owned(),
            main_content: vec![
                "article header", ".content__article-body"
            ],
            main_content_cleanup: vec![
                ".hide-on-mobile", ".inline-icon",
                ".atom__button", "input",
                ".meta__extras", ".content__headline-showcase-wrapper",
                ".fc-container__header",
                "figure.element-embed",
                ".vjs-control-text",
            ],
            delazify: true,
            fix_embeds: true,
            content_script: Some(r#"<script>
            [...document.querySelectorAll("[data-src-background]")]
                .map(d => d.src = d.dataset["srcBackground"].replace("background-image: url", "").replace(/[\(\)]/g, ""))
            </script>"#.to_owned()),
            preprocess: vec![
                AttributeRewrite {
                    selector: ".vjs-big-play-button[style]".to_owned(),
                    attribute: "style".to_owned(),
                    to_attribute: "data-src-background".to_owned(),
                    element_name: "img".to_owned()
                }
            ],
        });

        configs.insert("washingtonpost.com",
        SiteConfiguration {
            domain: "washingtonpost.com".to_owned(),
            main_content: vec![
                "main > header",
                "main > article .byline",
                "main > article [data-qa=\"timestamp\"]",
                "main > article figure",
                ".article-body",
                ".ent-article-body",
                "[data-feature-name^=\"etv3\"]",
            ],
            main_content_cleanup: vec![
                "header > nav",
                ".tooltip",
                "[data-qa=\"article-body-ad\"]",
            ],
            delazify: true,
            fix_embeds: false,
            content_script: None,
            preprocess: vec![
                AttributeRewrite {
                    selector: "[data-fallback-image-url]".to_owned(),
                    attribute: "data-fallback-image-url".to_owned(),
                    to_attribute: "src".to_owned(),
                    element_name: "img".to_owned()
                }
            ],
        });

        configs.insert("foxnews.com",
        SiteConfiguration {
            domain: "foxnews.com".to_owned(),
            main_content: vec![
                "article",
            ],
            main_content_cleanup: vec![
                ".sidebar", ".article-social", ".author-headshot"
            ],
            delazify: true,
            fix_embeds: false,
            content_script: None,
            preprocess: vec![],
        });

        configs.insert("forbes.com",
        SiteConfiguration {
            domain: "forbes.com".to_owned(),
            main_content: vec![
                "article > main",
                ".body-container"
            ],
            main_content_cleanup: vec![
                ".article-footer", ".disqus-module",
                ".article-sharing", "sharing",
                ".fs-author-avatar", ".fs-icon",
                ".contrib-bio button", ".contrib-bio .contributor-about__initial-description",
                "fbs-ad",
                "#speechkit-io-iframe",
            ],
            delazify: true,
            fix_embeds: false,
            content_script: None,
            preprocess: vec![],
        });

        configs.insert("cnbc.com",
        SiteConfiguration {
            domain: "cnbc.com".to_owned(),
            main_content: vec![
                "#main-article-header",
                "[data-module=\"ArticleBody\"]",
            ],
            main_content_cleanup: vec![
                ".InlineVideo-videoEmbed"
            ],
            delazify: false,
            fix_embeds: false,
            content_script: Some(r#"<script>
              [...document.querySelectorAll("figure")].map(f => {
                let imgid = f.id.replace("ArticleBody-InlineImage-", "");
                f.querySelector("img").src = "https://image.cnbcfm.com/api/v1/image/"+imgid+"-.jpeg?w=678&h=381";
              })
            </script>"#.to_owned()),
            preprocess: vec![
                AttributeRewrite {
                    selector: "[id^=\"ArticleBody-InlineImage\"]".to_owned(),
                    attribute: "id".to_owned(),
                    to_attribute: "id".to_owned(),
                    element_name: "figure".to_owned()
                },
                AttributeRewrite {
                    selector: "[id^=\"ArticleBody-InlineImage\"] .lazyload-placeholder".to_owned(),
                    attribute: "class".to_owned(),
                    to_attribute: "class".to_owned(),
                    element_name: "img".to_owned()
                },
                AttributeRewrite {
                    selector: "[id^=\"ArticleBody-InlineImage\"] > div > div:not([class*=\"imagePlaceholder\"])".to_owned(),
                    attribute: "id".to_owned(),
                    to_attribute: "id".to_owned(),
                    element_name: "figcaption".to_owned()
                }
            ],
        });

        configs.insert("usatoday.com",
        SiteConfiguration {
            domain: "usatoday.com".to_owned(),
            main_content: vec![
                "article",
               ".article-wrapper"
            ],
            main_content_cleanup: vec![
                ".gnt_ss", "svg", "custom-style"
            ],
            delazify: true,
            fix_embeds: false,
            content_script: None,
            preprocess: vec![
                AttributeRewrite {
                    selector: "button[data-c-vpattrs]".to_owned(),
                    attribute: "id".to_owned(),
                    to_attribute: "id".to_owned(),
                    element_name: "div".to_owned()
                },
                AttributeRewrite {
                    selector: "slide".to_owned(),
                    attribute: "original".to_owned(),
                    to_attribute: "src".to_owned(),
                    element_name: "img".to_owned()
                }
            ],
        });

        configs.insert("wsj.com",
        SiteConfiguration {
            domain: "wsj.com".to_owned(),
            main_content: vec![
                "article > main",
            ],
            main_content_cleanup: vec![
                "#saving-united-coupon-list", ".author-info"
            ],
            delazify: true,
            fix_embeds: false,
            content_script: None,
            preprocess: vec![],
        });

        configs.insert("reuters.com",
        SiteConfiguration {
            domain: "reuters.com".to_owned(),
            main_content: vec![
                ".ArticleHeader_container", ".StandardArticleBody_body"
            ],
            main_content_cleanup: vec![
                ".SmallImage_small-image", "[class$=expand-button]", ".Slideshow_caption", "[role=button]"
            ],
            delazify: true,
            fix_embeds: false,
            content_script: Some(r#"<script>
                [...document.querySelectorAll(".LazyImage_container img")].map(i => i.src = i.src.replace(/\&w=\d+/, "&w=600"));
            </script>"#.to_owned()),
            preprocess: vec![],
        });

        configs.insert("nypost.com",
        SiteConfiguration {
            domain: "nypost.com".to_owned(),
            main_content: vec![
                ".article-header", ".slide"
            ],
            main_content_cleanup: vec![
                ".no-mobile", ".author-contact", ".sharedaddy", ".author-flyout"
            ],
            delazify: true,
            fix_embeds: false,
            content_script: None,
            preprocess: vec![],
        });

        configs.insert("chron.com",
        SiteConfiguration {
            domain: "chron.com".to_owned(),
            main_content: vec![
                ".article-title", ".article-body"
            ],
            main_content_cleanup: vec![
                ".hidden", ".control-panel", ".article-body > script",
                ".caption-truncated"
            ],
            delazify: true,
            fix_embeds: false,
            content_script: None,
            preprocess: vec![
                AttributeRewrite {
                    selector: "li.hst-resgalleryitem".to_owned(),
                    attribute: "id".to_owned(),
                    to_attribute: "id".to_owned(),
                    element_name: "div".to_owned()
                }
            ],
        });

        configs.insert("nbcnews.com",
        SiteConfiguration {
            domain: "nbcnews.com".to_owned(),
            main_content: vec![
                ".article header", ".article article", ".article figure"
            ],
            main_content_cleanup: vec![
                ".article article svg", "[data-test=newsletter-signup]", "#emailSignup", ".ad-container"
            ],
            delazify: true,
            fix_embeds: false,
            content_script: None,
            preprocess: vec![],
        });

        configs.insert("dw.com",
        SiteConfiguration {
            domain: "dw.com".to_owned(),
            main_content: vec![
                "#bodyContent"
            ],
            main_content_cleanup: vec![
                "[class$=Teaser]", ".video", ".relatedContent", ".smallList", "#sharing-bar"
            ],
            delazify: true,
            fix_embeds: false,
            content_script: None,
            preprocess: vec![],
        });

        configs.insert("time.com",
        SiteConfiguration {
            domain: "time.com".to_owned(),
            main_content: vec!["main.article"],
            main_content_cleanup: vec![
                ".edit-link",
                ".most-popular-feed",
                ".inline-recirc",
                ".newsletter-callout",
                ".article-bottom",
                ".article-small-sidebar",
                ".ad",
                ".component.video video:not([poster])"
            ],
            delazify: true,
            fix_embeds: false,
            content_script: None,
            preprocess: vec![
                AttributeRewrite {
                    selector: "noscript".to_owned(),
                    attribute: "id".to_owned(),
                    to_attribute: "id".to_owned(),
                    element_name: "div".to_owned()
                }
            ],
        });

        configs.insert("cbsnews.com",
        SiteConfiguration {
            domain: "cbsnews.com".to_owned(),
            main_content: vec!["article.content", "article.article"],
            main_content_cleanup: vec![
                ".sharebar",
                ".content__cta",
                "figure .embed__content--draggable",
                "figure svg",
                "script",
                "[data-component=socialLinks]",
                "[data-component=sharebar]",
            ],
            delazify: true,
            fix_embeds: false,
            content_script: None,
            preprocess: vec![
                AttributeRewrite {
                    selector: "link[as=\"image\"]".to_owned(),
                    attribute: "href".to_owned(),
                    to_attribute: "src".to_owned(),
                    element_name: "img".to_owned()
                }
            ],
        });

        configs.insert("thedailybeast.com",
        SiteConfiguration {
            domain: "thedailybeast.com".to_owned(),
            main_content: vec!["article.Story", "body > div > script:not([src]):not([type])"],
            main_content_cleanup: vec![
                ".StandardHeader__share-buttons",
                ".StoryFooter",
                ".PullQuote__logo-icon",
                ".PullQuote__top-line",
                ".PullQuote__big-quote",
                "figure svg",
                ".SimpleAd",
                ".Byline__photo-link",
            ],
            delazify: true,
            fix_embeds: false,
            content_script: Some(r#"<script>
            [...document.querySelectorAll(".Body .LazyLoad")]
            .map((div, i) => {
                let lazyLoad = window.__INITIAL_STATE__.body.cards
                    .filter(c => c[0] === "pt-image" || c[0] === "pt-video-card")[i];
                if (!lazyLoad || lazyLoad[0] !== "pt-image") return;
                let figure = document.createElement("figure");
                figure.innerHTML = '<img src="https://img.thedailybeast.com/image/upload/c_crop/dpr_1.5/c_limit,w_800/fl_lossy,q_auto/'+lazyLoad[1].public_id+'"><figcaption>'+lazyLoad[1].title+' Credit: '+lazyLoad[1].credit+'</figcaption>';
                div.appendChild(figure);
                console.log(div, lazyLoad);
            })
            </script>"#.to_owned()),
            preprocess: vec![
                AttributeRewrite {
                    selector: ".PullQuote".to_owned(),
                    attribute: "id".to_owned(),
                    to_attribute: "id".to_owned(),
                    element_name: "blockquote".to_owned()
                }
            ],
        });

        configs.insert("businessinsider.com",
        SiteConfiguration {
            domain: "businessinsider.com".to_owned(),
            main_content: vec![
                ".post-headline", ".byline-wrapper",
                "#l-content", ".container figure",
            ],
            main_content_cleanup: vec![
                ".share-wrapper", ".ad",
                ".category-tagline",
                ".popular-video",
                "figure .lazy-image", "figure .lazy-blur",
            ],
            delazify: true,
            fix_embeds: false,
            content_script: None,
            preprocess: vec![
                AttributeRewrite {
                    selector: "figure noscript".to_owned(),
                    attribute: "id".to_owned(),
                    to_attribute: "id".to_owned(),
                    element_name: "div".to_owned()
                }
            ],
        });

        ()
    }
}
