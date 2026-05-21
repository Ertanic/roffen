import {propertiesBody} from "./common.ts";
import {markDirty} from "./save-content.ts";
import type {
    IComponentInitPropsContext,
    IComponentMountContext,
    IComponentNormalizeContext
} from "common/src/contexts.ts";

type DataMap = Record<string, string>;

export class ComponentMountContext implements IComponentMountContext {
    private el: HTMLElement;
    private html: string;
    private data: DataMap;
    private title: string;

    constructor(el: HTMLElement, html: string, data: DataMap, title: string) {
        this.el = el;
        this.html = html;
        this.data = data;
        this.title = title;
    }
}

export class ComponentSetSizeContext {
    private el: HTMLElement;
    private data: DataMap;

    constructor(el: HTMLElement, data: DataMap) {
        this.el = el;
        this.data = data;
    }

    setRow(size: number) {
        this.el.style.gridRow = `span ${size}`;
        this.el.dataset.row = size.toString();
    }

    setCol(size: number) {
        this.el.style.gridColumn = `span ${size}`;
        this.el.dataset.col = size.toString();
    }
}

export class ComponentInitPropsContext implements IComponentInitPropsContext {
    public el: HTMLElement;
    public data: DataMap;
    public panel: HTMLDivElement;
    public refs: number;

    constructor(el: HTMLElement, data: DataMap, panel: HTMLDivElement) {
        this.el = el;
        this.data = data;
        this.panel = panel;
        this.refs = 0;
    }

    cancelAnimationFrame(n: number): void {
        cancelAnimationFrame(n);
    }

    requestAnimationFrame(callback: () => void): void {
        this.refs = requestAnimationFrame(callback);
    }

    keepBlockInView(callback: () => void) {
        const before = this.el.getBoundingClientRect().top;

        callback();

        const after = this.el.getBoundingClientRect().top;

        const canvas = document.querySelector(".editor-canvas");
        if (canvas) {
            canvas.scrollTop += (after - before);
        }
    }

    autoResizeRows() {
        const rowHeight = 80;
        this.el.style.gridRow = "span 1";

        const child = this.el.children[0];
        if (child) {
            const contentHeight = child.scrollHeight;
            const rows = Math.max(1, Math.ceil(contentHeight / rowHeight));
            this.applyRows(rows);
        }
    }

    applyRows(rows: number) {
        const value = Math.max(1, rows);

        this.el.dataset.row = String(value);
        this.el.style.gridRow = `span ${value}`;

        this.syncProperties();
        markDirty();
    }

    syncProperties() {
        if (!this.el.classList.contains("selected")) return;

        const rowInput = propertiesBody?.querySelector<HTMLInputElement>('#height-prop');
        if (rowInput && this.el.dataset.row) {
            rowInput.value = this.el.dataset.row;
        }
    }

    createTextarea(label: string, defaultVal: string, onChange: (val: string) => void) {
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
        this.panel.appendChild(wrap);
        return wrap;
    }

    createNumber(label: string, defaultVal: string, min: string, max: string, onChange: (val: string) => void) {
        const wrap = document.createElement("div");
        wrap.className = "property";

        const l = document.createElement("label");
        l.textContent = label;

        const input = document.createElement("input");
        input.type = "number";
        input.min = min.toString();
        input.max = max.toString();
        input.value = defaultVal.toString();

        input.addEventListener("input", e => {
            onChange((e.target as HTMLInputElement).value);
            markDirty();
        });

        wrap.append(l, input);
        this.panel.appendChild(wrap);
        return wrap;
    }

    createUrlSource(label: string, value: string, onChange: (val: string) => void) {
        const wrap = document.createElement("div");
        wrap.className = "property";

        const l = document.createElement("label");
        l.textContent = label;

        const input = document.createElement("input");
        input.type = "url";
        input.value = value;

        input.addEventListener("input", e => {
            onChange((e.target as HTMLInputElement).value);
            markDirty();
        });

        wrap.append(l, input);
        this.panel.appendChild(wrap);
        return wrap;
    }
}

export class ComponentFetchDataContext {
    private el: HTMLElement;
    private data: DataMap;

    constructor(el: HTMLElement, data: DataMap) {
        this.el = el;
        this.data = data;
    }
}

export class ComponentNormalizeContext implements IComponentNormalizeContext {
    public el: HTMLElement;
    public data: DataMap;

    constructor(el: HTMLElement, data: DataMap) {
        this.el = el;
        this.data = data;
    }
}