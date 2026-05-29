import {componentsCache, propertiesBody} from "./common.ts";
import type {Component} from "common/src/Component.ts";
import {markDirty} from "./save-content.ts";
import {defaultReactiveState, initProps, initReactiveHTML, type ReactiveState} from "./reactive.ts";

export function initProperties(block: HTMLDivElement, comp: Component | null = null, state: ReactiveState | null) {
    block.addEventListener("click", () => {
        document.querySelectorAll(".grid-block.selected")
            .forEach(el => el.classList.remove("selected"));

        block.classList.add("selected");

        if (!propertiesBody) {
            console.error("no properties body found");
            return;
        }

        propertiesBody.innerHTML = "";

        if (!comp) {
            const type = block.dataset.type;
            if (!type) {
                console.error("no block type found")
                return;
            }

            const component = componentsCache.get(type);
            if (!component) {
                console.error("no component found for type: " + block.dataset.type);
                return;
            }
            comp = component;
        }

        if (!state) {
            state = initProps(comp.properties, defaultReactiveState(), block);
            initReactiveHTML(comp.html, state, block);
        }

        for (const prop of comp.properties) {
            switch (prop.type_name) {
                case "Number":
                    createNumber(
                        propertiesBody,
                        prop.name,
                        block.dataset[prop.name] ?? prop.default ?? "0",
                        prop.min,
                        prop.max,
                        value => {
                            state![prop.name]?.next(value);
                            block.dataset[prop.name] = value;
                            markDirty();
                        }
                    )
                    break;
                case null:
                case undefined:
                case "String":
                    createText(
                        propertiesBody,
                        prop.name,
                        block.dataset[prop.name] ?? prop.default ?? "",
                        value => {
                            state![prop.name]?.next(value);
                            block.dataset[prop.name] = value;
                            markDirty();
                        }
                    )
                    break;
            }
        }

        /* GRID WIDTH */
        createNumber(
            propertiesBody,
            "Width (columns)",
            block.dataset.col ?? "12",
            "1", "12",
            value => {
                block.dataset.col = value;
                block.style.gridColumn = `span ${value}`;
            }
        );

        /* GRID HEIGHT */
        createNumber(
            propertiesBody,
            "Height (rows)",
            block.dataset.row ?? "1",
            "1", "",
            value => {
                block.dataset.row = value;
                block.style.gridRow = `span ${value}`;
            }
        );
    });
}

function createText(panel: HTMLElement, label: string, defaultVal: string, onChange: (value: string) => void) {
    const wrap = document.createElement("div");
    wrap.className = "property";

    const l = document.createElement("label");
    l.textContent = label;

    const ta = document.createElement("textarea");
    ta.value = defaultVal;

    ta.addEventListener("input", e => {
        onChange((e.currentTarget as HTMLTextAreaElement).value);
        markDirty();
    });

    wrap.append(l, ta);
    panel.appendChild(wrap);
}

function createNumber(panel: HTMLElement, label: string, defaultVal: string | undefined, min: string | undefined, max: string | undefined, onChange: (value: string) => void) {
    const wrap = document.createElement("div");
    wrap.className = "property";

    const l = document.createElement("label");
    l.textContent = label;

    const ta = document.createElement("input");
    ta.type = "number";
    ta.min = min ?? "";
    ta.max = max ?? "";
    ta.value = defaultVal ?? "0";

    ta.addEventListener("input", e => {
        onChange((e.currentTarget as HTMLTextAreaElement).value);
        markDirty();
    });

    wrap.append(l, ta);
    panel.appendChild(wrap);
}