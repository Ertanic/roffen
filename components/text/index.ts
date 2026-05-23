import type {IComponentHooks} from "common/src/IComponent.ts";

export const hooks: IComponentHooks = {
    initProps: ctx => {
        ctx.createTextarea("Content",
            ctx.data.content ?? ctx.el.innerText,
            value => {
                ctx.cancelAnimationFrame(ctx.refs)
                ctx.requestAnimationFrame(() => {
                    ctx.keepBlockInView(() => {
                        ctx.data.content = value;
                        (ctx.el.children[0] as HTMLParagraphElement).innerText = value;
                        ctx.autoResizeRows();
                    })
                });
            });
    },

    mount: ctx => {
        const textEl = document.createElement("p");
        textEl.innerText = ctx.data.content ?? ctx.el.innerText;
        ctx.el.appendChild(textEl);
    },

    setSize: ctx => {
        if (!ctx.data.col) {
            console.log("no column size found in component");
            return;
        }
        ctx.setCol(Number(ctx.data.col));
    },

    fetchData: ctx => {
        return {
            content: (ctx.el.children[0] as HTMLParagraphElement)?.innerText ?? ctx.data.content,
        }
    }
}