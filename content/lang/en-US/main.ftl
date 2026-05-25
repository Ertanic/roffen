main-page-contents = Contents
main-page-about = About
main-page-about-paragraph-1 = Roffen is a little file-flat CMS for your personal blog. You can use it to create and manage your blog posts, and it will help you to keep track of your content.
main-page-about-paragraph-2 = It means that you can easily create or modify an existing file so that the changes are applied in real time.
main-page-about-paragraph-3 = Alternatively, you can use the interactive article editor in the <a href="/admin/posts">admin panel</a>.
main-page-about-paragraph-4 = The pages editor is in development. You can follow the development process in the <a href="https://github.com/Ertanic/roffen">repository</a>.
main-page-plans = Another plans
main-page-plan-1 = translating
main-page-plan-2 = adding more components
main-page-plan-3 = adding themes support

main-page-pages = Pages
main-page-pages-paragraph-1 = Since the page editor is still under development, you will have to create pages manually.
main-page-pages-paragraph-2 = To do this, create an <code>index.html</code> file in <code>content/pages/</code>. This action will override the existing main page. To create or override a subpage like <code>/posts</code>, create an <code>index.html</code> file in <code>content/pages/posts/</code>.

main-page-templates = Templates
main-page-templates-paragraph-1 = The engine uses upon for page templates, which is a simple yet powerful engine. You can read more about it in its <a href="https://docs.rs/upon/latest/upon/">documentation</a>.
main-page-templates-paragraph-2 = Let's talk about the functions that are implemented and registered in the engine.
main-page-template-function-1 = checks if the current value is a map.
main-page-template-function-2 = get the length of the current value.
main-page-template-function-3 = formats a timestamp into a date string.
main-page-template-function-4 = checks if two values are equal.
main-page-template-function-5 = checks if two values are both true.
main-page-template-function-6 = returns the default value if the current value is empty.
main-page-template-function-7 = returns all posts.
main-page-template-function-8 = returns a list of posts.
main-page-template-function-9 = returns a post by its ID.
main-page-template-function-10 = returns all pages.
main-page-template-values = In addition to functions, the following values are passed to templates:
main-page-template-value-1 = authentication status.
main-page-template-value-2 = query parameters.
main-page-template-quote = If you don't have any functions or values in the template engine, you can always manually add them. See this <a href="https://github.com/Ertanic/roffen/blob/f60b9db7f066b3ba674662b2263b1770ed2c3ccf/server/src/templates/functions.rs">file</a> and the project <a href="#Build">build</a> method.
main-page-template-structure = Let's now look at the fields of structures that are returned from functions and constants.

main-page-components = Components
main-page-components-paragraph-1 = The engine is somewhat modular, which means that you can write your own components, connect them to the engine, and use them in your own articles.
main-page-components-paragraph-2 = Each module must contain/compile <code>index.js</code> and <code>meta.ron</code> and be located in the corresponding directory under <code>content/components/</code>.
main-page-components-lifecycle = First, let's explore the component's life cycle. In general, everything is clear here, except for the difference between <code>mount</code> and <code>normalize</code>. The only difference is that <code>mount</code> is triggered by a drag event, while <code>normalize</code> is triggered by page loading and converts the component according to its properties.
main-page-components-implementation = Implementation
main-page-components-implementation-paragraph = Let's try to implement the image component. To do this, create a project with the following content:
main-page-components-use = Now we can use this component in the editor.
main-page-components-rendering = Rendering
main-page-components-rendering-paragraph = There are several things you need to do to render components on a page:
main-page-components-rendering-step-1 = Add the required styles to the page.
main-page-components-rendering-step-2 = Next, you need to add the following script to your page.
main-page-components-rendering-step-3 = Finally, you can use components by generating their data containers inside a container with the <code>.grid-content</code> class.
main-page-components-rendering-step-4 = To view the results of the script, see the <a href="/posts">Posts</a> page.
main-page-components-why = Why
main-page-components-why-paragraph = Why such difficulties? At first, I planned to make components using an embedded language, but I quickly abandoned this idea because it would have complicated interaction with the DOM tree. Therefore, I had to use native JS/TS scripts to give you complete freedom of action.

main-page-build = Build
main-page-build-paragraph-1 = To build the project, you will need several things:
main-page-build-paragraph-2 = If everything is installed, you can use the following command, which will compile both the server and all the typescript code in release mode, optimizing the entire code.
main-page-build-paragraph-3 = Upon successful build, the executable file will appear in the <code>target/release/server(.exe)</code> path.
main-page-build-paragraph-4 = That's all, you won't need anything else in this directory, all resources from the <code>content/</code> folder are included in the output executable file by default.
main-page-build-debug = In debug mode, the project can be started with the following command. In this case, no optimizations or content inclusion will be applied, so you can safely change the contents of this folder in live mode.