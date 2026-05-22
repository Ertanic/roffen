import type {IComponentHooks} from "common/src/IComponent.ts";

export const hooks: IComponentHooks = {
    mount: ctx => {
        const previewEl = document.createElement("img");
        const source = ctx.data.source;

        if (!source) {
            console.error("no source link");
            return;
        }

        previewEl.src = source;

        ctx.el.classList.add("component-image");

        ctx.el.appendChild(previewEl);
    },
    setSize: ctx => {
        if (!ctx.data.row || !ctx.data.col) {
            console.warn("no row or column parameter in component");
            return;
        }

        ctx.setRow(Number(ctx.data.row));
        ctx.setCol(Number(ctx.data.col));
    },
    initProps: ctx => {
        ctx.createUrlSource(
            "Image url",
            ctx.data.source ?? (ctx.el.children.item(0) as HTMLImageElement)?.src,
            value => {
                const child = ctx.el.children.item(0) as HTMLImageElement;

                if (!child) {
                    console.error("no child element");
                    return;
                }

                if (value) {
                    child.src = value;
                    ctx.data.source = value;
                } else if (ctx.data.source) {
                    child.src = ctx.data.source;
                }
            });
    },
    fetchData: ctx => {
        const child = ctx.el.children.item(0) as HTMLImageElement;

        if (!child) {
            return {
                source: "",
            };
        } else {
            return {
                source: child.src,
            }
        }
    }
}