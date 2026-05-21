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
    }
}