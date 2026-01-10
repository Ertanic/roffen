const quoteComponent = {
    initProps: ctx => {
        ctx.createTextarea("Content",
            ctx.data.content ?? ctx.el.innerText,
            value => {
                cancelAnimationFrame(ctx.refs.content)
                ctx.refs.content = requestAnimationFrame(() => {
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