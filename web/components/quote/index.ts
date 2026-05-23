import type {IComponentHooks} from "common/src/IComponent.ts";

export const hooks: IComponentHooks = {
    initProps: ctx => {
        ctx.createTextarea("Content",
            ctx.data.content ?? ctx.el.innerText,
            value => {
                ctx.cancelAnimationFrame(ctx.refs)
                ctx.requestAnimationFrame(() => {
                    ctx.keepBlockInView(() => {
                        ctx.el.children[0].innerText = value;
                        ctx.autoResizeRows();
                    })
                });
            });
    },

    fetchData: ctx => {
        return {
            content: ctx.el.children[0].innerText,
        }
    },

    mount: ctx => {
        const quote = document.createElement('blockquote');
        quote.innerText = ctx.data.content ?? ctx.el.innerText;
        ctx.el.appendChild(quote);
    },

    setSize: ctx => {
        ctx.setRow(Number(ctx.data.row));
        ctx.setCol(Number(ctx.data.col));
    }
}