
declare namespace SpeedReader {

    interface SpeedReaderConfig {
        domain: string;
        url_rules: string[];
        declarative_rewrite?: RewriteRules;
    }

    interface RewriteRules {
        main_content: string[];
        main_content_cleanup: string[];
        delazify: boolean;
        fix_embeds: boolean;
        content_script?: string;
        preprocess: AttributeRewrite[];
    }

    interface AttributeRewrite {
        selector: string;
        attribute?: [string?, string?];
        element_name: string;
    }

}
