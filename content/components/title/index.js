const titleComponent = {
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

        ctx.createNumber(
            "Level",
            ctx.data.level ?? 1,
            1,
            6,
            value => {
                ctx.el.innerHTML = `<h${value}>${ctx.el.children[0].innerText}</h${value}>`;
                ctx.el.dataset.level = value;
            });
    },
    fetchData: ctx => {
        return {
            content: ctx.el.children[0].innerText,
            level: ctx.el.dataset.level,
        }
    }
}