
declare namespace SpeedReader {

    type Configuration = SpeedReaderConfig[];

    interface SpeedReaderConfig {
        domain: string;
        /**
         * Adblock-style rules matching URLs to run SpeedReader on
         *
         * @items.type string
         * @items.minimum 1
         */
        url_rules: string[];
        /** @nullable */
        declarative_rewrite?: RewriteRules;
    }

    interface RewriteRules {
        main_content: string[];
        main_content_cleanup: string[];
        delazify: boolean;
        fix_embeds: boolean;
        /** @nullable */
        content_script?: string;
        preprocess: AttributeRewrite[];
    }

    interface AttributeRewrite {
        selector: string;
        /** @nullable */
        attribute?: [string?, string?];
        element_name: string;
    }

}
