const imageComponent = {
    mount: ctx => {
        const previewEl = document.createElement("img");
        previewEl.src = ctx.data.source;

        ctx.el.classList.add("component-image");

        ctx.el.appendChild(previewEl);
    },
    setSize: ctx => {
        ctx.setRow(ctx.data.row);
        ctx.setCol(ctx.data.col);
    },
    initProps: ctx => {
        ctx.createUrlSource(
            "Image url",
            "",
            value => {
                if (value) {
                    ctx.el.children[0].src = value;
                } else {
                    ctx.el.children[0].src = ctx.data.source;
                }
            });
    },
    fetchData: ctx => {
        return {
            source: ctx.el.children[0].src,
        }
    }
}