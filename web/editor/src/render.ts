import type {ComponentHtml, ComponentProperty} from "common/src/Component.ts";
import {
    defaultReactiveState,
    initAttributeBindings,
    initChildrenBindings,
    initProps, initTagBinding,
    type ReactiveState
} from "./reactive.ts";

export type Binding = {
    type: "interactive" | "content",
    content: string,
    properties?: string[]
}

export function renderHTML(html: ComponentHtml, props: ComponentProperty[]): { html: Node, state: ReactiveState } {
    const state = initProps(props, defaultReactiveState(), null);

    const el = renderElement(html, state);

    return {html: el, state};
}

function renderElement(html: ComponentHtml, state: ReactiveState): Node {
    const binding = parseBinding(html.element, state);

    const element = document.createElement(binding.content);
    let ref = {
        current: element
    };

    const children = (html.children ?? []).map(child => {
        if (child.type === "content") {
            return document.createTextNode(child.content);
        } else {
            return renderElement(child.content, state);
        }
    });

    ref.current.append(...children);

    html.children?.forEach((child, i) => initChildrenBindings(child, ref, state))

    initAttributeBindings(html, ref, state);

    if (binding.type === "interactive") {
        initTagBinding(html, ref, state, binding);
    }

    return ref.current;
}

export function parseBinding(value: string, state: ReactiveState): Binding {
    const matches = value.match(/\#\(?(\w+)\)?/);
    if (matches) {
        let result = value;
        const props = [];
        for (const match of matches) {
            if (state[match]) {
                result = result.replace(/(\#\(?\w+\)?)/, state[match].getValue());
                props.push(match);
            }
        }
        return {type: 'interactive', content: result, properties: props};
    } else {
        return {type: "content", content: value};
    }
}