export type ComponentPropertyType = "String" | "Number" | "Boolean";

export type ComponentProperty = {
    name: string;
    lang_key: string;
    default?: string,
    max?: string,
    min?: string,
    type_name?: ComponentPropertyType,
};

export type ComponentHtmlChild = {
    type: "content";
    content: string;
} | {
    type: "html";
    content: ComponentHtml;
}

export type ComponentHtml = {
    element: string;
    attributes: Record<string, string>;
    children: ComponentHtmlChild[];
};

export type Component = {
    name: string;
    html: ComponentHtml;
    properties: ComponentProperty[];
};