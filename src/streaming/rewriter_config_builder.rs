use lol_html::html_content::*;
use lol_html::ElementContentHandlers;
use lol_html::Selector;
use std::error::Error;

pub type HandlerResult = Result<(), Box<dyn Error>>;
pub type ElementHandler = Box<dyn Fn(&mut Element) -> HandlerResult>;
pub type TextHandler = Box<dyn Fn(&mut TextChunk) -> HandlerResult>;


#[derive(Clone, Debug)]
pub struct AttributeRewrite {
    pub selector: String,
    pub attribute: String,
    pub to_attribute: String,
    pub element_name: String,
}

#[derive(Clone, Debug)]
pub struct SiteConfiguration {
    pub domain: String,
    pub main_content: Vec<String>,
    pub main_content_cleanup: Vec<String>,
    pub delazify: bool,
    pub fix_embeds: bool,
    pub content_script: Option<String>,
    pub preprocess: Vec<AttributeRewrite>,
}

impl SiteConfiguration {
    pub fn get_main_content_selectors(&self) -> Vec<&str> {
        self.main_content.iter().map(AsRef::as_ref).collect()
    }
    pub fn get_content_cleanup_selectors(&self) -> Vec<&str> {
        self.main_content_cleanup.iter().map(AsRef::as_ref).collect()
    }
}

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
pub fn get_content_handlers<'h>(function: &'h ContentFunction) -> ElementContentHandlers<'h> {
    if function.element.is_some() {
        ElementContentHandlers::default().element(function.element.as_ref().unwrap())
    } else if function.text.is_some() {
        ElementContentHandlers::default().text(function.text.as_ref().unwrap())
    } else {
        ElementContentHandlers::default()
    }
}

pub struct RewriterConfigBuilder {
    pub handlers: Vec<(Selector, ContentFunction)>,
}

impl RewriterConfigBuilder {
    pub fn new(conf: &SiteConfiguration, origin: &str) -> Self {
        let mut element_content_handlers = vec![];

        for attr_rewrite in &conf.preprocess {
            let rewrite = attr_rewrite.clone();
            add_element_function(
                &mut element_content_handlers,
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

        collect_main_content(
            &mut element_content_handlers,
            &conf.get_main_content_selectors(),
            &conf.get_content_cleanup_selectors(),
        );
        if conf.delazify {
            delazify(&mut element_content_handlers);
        }
        if conf.fix_embeds {
            fix_social_embeds(&mut element_content_handlers);
        }
        correct_relative_links(&mut element_content_handlers, origin.to_owned());

        let maybe_script = conf.content_script.clone();
        maybe_script.map(|script| {
            add_element_function(
                &mut element_content_handlers,
                "body",
                Box::new(move |el| {
                    el.append(&script, ContentType::Html);
                    Ok(())
                }),
            )
        });

        RewriterConfigBuilder {
            handlers: element_content_handlers,
        }
    }
}

fn add_element_function(
    handlers: &mut Vec<(Selector, ContentFunction)>,
    selector: &str,
    handler: ElementHandler,
) {
    handlers.push((
        selector.parse::<Selector>().unwrap(),
        ContentFunction::from(handler),
    ));
}

fn add_text_function(
    handlers: &mut Vec<(Selector, ContentFunction)>,
    selector: &str,
    handler: TextHandler,
) {
    handlers.push((
        selector.parse::<Selector>().unwrap(),
        ContentFunction::from(handler),
    ))
}

fn collect_main_content(
    handlers: &mut Vec<(Selector, ContentFunction)>,
    content_selectors: &[&str],
    cleanup_selectors: &[&str],
) {
    content_selectors.iter().for_each(|selector| {
        add_element_function(
            handlers,
            &format!("{}", selector),
            Box::new(mark_retained_element),
        );
        add_text_function(
            handlers,
            &format!("{}", selector),
            Box::new(mark_retained_text),
        );
        add_element_function(
            handlers,
            &format!("{} *", selector),
            Box::new(mark_retained_element),
        );
        add_text_function(
            handlers,
            &format!("{} *", selector),
            Box::new(mark_retained_text),
        );
    });

    cleanup_selectors.iter().for_each(|selector| {
        add_element_function(handlers, selector, Box::new(|el| Ok(el.remove())));
    });

    // Drop everything else
    add_text_function(handlers, "*", Box::new(remove_unmarked_text));
    add_element_function(handlers, "*", Box::new(unwrap_unmarked_element));
    add_element_function(
        handlers,
        "[style]",
        Box::new(|el| Ok(el.remove_attribute("style"))),
    );
}

fn correct_relative_links(handlers: &mut Vec<(Selector, ContentFunction)>, origin: String) {
    let href_origin = origin.clone();
    add_element_function(
        handlers,
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
    add_element_function(
        handlers,
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

fn delazify(handlers: &mut Vec<(Selector, ContentFunction)>) {
    add_element_function(
        handlers,
        "[data-src]",
        Box::new(|el| {
            el.get_attribute("data-src").map(|src| {
                el.set_attribute("src", &src).ok();
            });
            Ok(())
        }),
    );
    add_element_function(
        handlers,
        "[data-srcset]",
        Box::new(|el| {
            el.get_attribute("data-srcset").map(|srcset| {
                el.set_attribute("srcset", &srcset).ok();
            });
            Ok(())
        }),
    );
    add_element_function(
        handlers,
        "[data-original]",
        Box::new(|el| {
            el.get_attribute("data-original").map(|original| {
                el.set_attribute("src", &original).ok();
            });
            Ok(())
        }),
    );
    add_element_function(
        handlers,
        "img[data-src-medium]",
        Box::new(|el| {
            el.get_attribute("data-src-medium").map(|original| {
                el.set_attribute("src", &original).ok();
            });
            Ok(())
        }),
    );
    add_element_function(
        handlers,
        "img[data-raw-src]",
        Box::new(|el| {
            el.get_attribute("data-raw-src").map(|original| {
                el.set_attribute("src", &original).ok();
            });
            Ok(())
        }),
    );
    add_element_function(
        handlers,
        "img[data-gl-src]",
        Box::new(|el| {
            el.get_attribute("data-gl-src").map(|original| {
                el.set_attribute("src", &original).ok();
            });
            Ok(())
        }),
    );
    add_element_function(
        handlers,
        "img",
        Box::new(|el| {
            el.remove_attribute("height");
            el.remove_attribute("width");
            Ok(())
        }),
    );
}

fn fix_social_embeds(handlers: &mut Vec<(Selector, ContentFunction)>) {
    add_element_function(
        handlers,
        ".twitterContainer",
        Box::new(|el: &mut Element| {
            el.prepend(r#"
            <script type="text/javascript" src="//platform.twitter.com/widgets.js" async="">
            </script>"#, ContentType::Html);
            Ok(())
        }),
    )
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