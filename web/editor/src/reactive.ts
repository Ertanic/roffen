import {BehaviorSubject} from "@reactivex/rxjs/src";
import type {ComponentHtml, ComponentHtmlChild, ComponentProperty} from "common/src/Component.ts";
import {type Binding, parseBinding} from "./render.ts";

type Ref = {
    current: HTMLElement
}

export type ReactiveState = {
    [k: string]: BehaviorSubject<any>
}

export function defaultReactiveState(): ReactiveState {
    return {
        col: new BehaviorSubject("12"),
        row: new BehaviorSubject("1"),
    };
}

export function initProps(props: ComponentProperty[], state: ReactiveState, block: HTMLElement | null): ReactiveState {
    for (const prop of props) {
        state[prop.name] = new BehaviorSubject(block?.dataset[prop.name] ?? prop.default ?? "");
    }
    return state;
}

export function initReactiveHTML(html: ComponentHtml, state: ReactiveState, block: HTMLDivElement) {
    if (!block.firstChild) {
        return;
    }

    const ref = {
        current: block.firstElementChild as HTMLElement
    };

    walk(html, ref, state)
}

export function initTagBinding(html: ComponentHtml, ref: Ref, state: ReactiveState, binding: Binding | null) {
    for (const prop of binding?.properties ?? []) {
        state[prop]?.subscribe(_ => {
            const old = ref.current;

            const newEl = document.createElement(parseBinding(html.element, state).content);

            newEl.append(...Array.from(old.childNodes));

            for (let attr of old.getAttributeNames()) {
                newEl.setAttribute(attr, old.getAttribute(attr) ?? "");
            }

            const parent = old.parentNode;
            if (parent) {
                console.log(parent)
                parent.insertBefore(newEl, old);
                parent.removeChild(old);
            }

            ref.current = newEl;
        });
    }
}

export function initAttributeBindings(html: ComponentHtml, ref: Ref, state: ReactiveState) {
    for (let key in html.attributes) {
        const val = html.attributes[key];
        if (!val) continue;

        const binding = parseBinding(val, state);
        if (binding.type === "content") {
            ref.current.setAttribute(key, binding.content);
        } else {
            ref.current.setAttribute(key, binding.content);
            for (const prop of binding.properties ?? []) {
                state[prop]?.subscribe(v => ref.current.setAttribute(parseBinding(key, state).content, v));
            }
        }
    }
}

export function initChildrenBindings(html: ComponentHtmlChild, ref: Ref, state: ReactiveState) {
    console.log("child: ", html, ref.current)
    if (ref.current instanceof HTMLElement && html.type === "html") {
        console.log("walk: ", html.content)
        walk(html.content, ref, state);
    } else if (html.type === "content") {
        const bindings = parseBinding(html.content, state);
        for (const prop of bindings.properties ?? []) {
            state[prop]?.subscribe(v => ref.current.textContent = parseBinding(html.content, state).content);
        }
    }
}

function walk(
    html: ComponentHtml,
    ref: Ref,
    state: ReactiveState
) {
    const old = ref.current;

    initTagBinding(html, ref, state, null);
    initAttributeBindings(html, ref, state);

    html.children.forEach((child, i) => {
        ref.current = ref.current.childNodes[i]! as HTMLElement;
        initChildrenBindings(child, ref, state);
        ref.current = old;
    });
}