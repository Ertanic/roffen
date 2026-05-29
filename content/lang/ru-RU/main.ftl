main-page-contents = Содержание
main-page-about = О проекте
main-page-about-paragraph-1 = Roffen — это небольшая flat-file CMS для вашего персонального блога. Вы можете использовать её для создания и управления своими постами, а также для удобного отслеживания контента.
main-page-about-paragraph-2 = Это означает, что вы можете легко создавать или изменять существующие файлы, а изменения будут применяться в реальном времени.
main-page-about-paragraph-3 = В качестве альтернативы вы можете использовать интерактивный редактор статей в <a href="/admin/posts">панели администратора</a>.
main-page-about-paragraph-4 = Редактор страниц находится в разработке. Вы можете следить за процессом разработки в <a href="https://github.com/Ertanic/roffen">репозитории</a>.
main-page-plans = Планы
main-page-plan-2 = добавление новых компонентов
main-page-plan-3 = поддержка тем

main-page-config = Конфигурация
main-page-config-auth = Авторизация
main-page-config-tls = TLS
main-page-config-server = Сервер
main-page-config-lang = Язык
main-page-config-default = По умолчанию

main-page-config-paragraph-1 = Файл конфигурации представляет собой файл <code>config.toml</code>, расположенный в корне папки content. Этот файл также поддерживает горячую перезагрузку, что позволяет изменять некоторые параметры на лету.
main-page-config-paragraph-2 = Если вы планируете изменить значения на нестандартные, конфигурация должна содержать следующее. В противном случае вы не сможете получить доступ к панели администратора.
main-page-config-paragraph-3 = Если вы планируете запускать сервер через HTTPS, необходимо указать пути к сертификатам сервера. Поддерживаются как относительные, так и абсолютные пути.
main-page-config-paragraph-4 = По умолчанию сервер слушает адрес <code>0.0.0.0</code>. Порт зависит от использования HTTPS: если указаны пути к сертификатам, используется порт <code>443</code>, иначе — <code>80</code>. Чтобы указать другой адрес или порт, можно использовать следующие значения в конфигурации.
main-page-config-paragraph-5 = Некоторые части интерфейса также поддерживают локализацию. Чтобы включить её, можно указать следующее:
main-page-config-paragraph-6 = Также существует настройка резервного языка, который будет использоваться системой, если перевод для текущего языка не найден.
main-page-config-paragraph-7 = По умолчанию конфигурация выглядит следующим образом. Вы можете скопировать её отсюда, если вам нужно изменить только несколько значений.

main-page-pages = Страницы
main-page-pages-paragraph-1 = Поскольку редактор страниц всё ещё находится в разработке, вам придётся создавать страницы вручную.
main-page-pages-paragraph-2 = Для этого создайте файл <code>index.html</code> в <code>content/pages/</code>. Это действие переопределит существующую главную страницу. Чтобы создать или переопределить подстраницу, например <code>/posts</code>, создайте файл <code>index.html</code> в <code>content/pages/posts/</code>.

main-page-templates = Шаблоны
main-page-templates-paragraph-1 = Для шаблонов страниц движок использует upon — простой, но мощный шаблонизатор. Подробнее о нём можно прочитать в его <a href="https://docs.rs/upon/latest/upon/">документации</a>.
main-page-templates-paragraph-2 = Давайте рассмотрим функции, реализованные и зарегистрированные в движке.
main-page-template-function-1 = проверяет, является ли текущее значение map-объектом.
main-page-template-function-2 = возвращает длину текущего значения.
main-page-template-function-3 = форматирует timestamp в строку даты.
main-page-template-function-4 = проверяет, равны ли два значения.
main-page-template-function-5 = проверяет, являются ли оба значения истинными.
main-page-template-function-6 = возвращает значение по умолчанию, если текущее значение пустое.
main-page-template-function-7 = возвращает все посты.
main-page-template-function-8 = возвращает список постов.
main-page-template-function-9 = возвращает пост по его ID.
main-page-template-function-10 = возвращает все страницы.
main-page-template-function-11 = возвращает все компоненты.
main-page-template-function-12 = рендерит компонент.
main-page-template-function-13 = возвращает перевод по fluent-ключу.
main-page-template-value-3 = параметры маршрута.
main-page-template-values = Помимо функций, в шаблоны также передаются следующие значения:
main-page-template-value-1 = статус аутентификации.
main-page-template-value-2 = параметры запроса.
main-page-template-quote = Если в шаблонном движке отсутствуют какие-либо функции или значения, вы всегда можете добавить их вручную. См. этот <a href="https://github.com/Ertanic/roffen/blob/f60b9db7f066b3ba674662b2263b1770ed2c3ccf/server/src/templates/functions.rs">файл</a> и метод <a href="#Build">сборки</a> проекта.
main-page-template-structure = Теперь давайте посмотрим на поля структур, возвращаемых функциями и константами.

main-page-components = Компоненты
main-page-components-paragraph-1 = Движок является модульным, что означает, что вы можете писать собственные компоненты, подключать их к движку и использовать в своих статьях.
main-page-components-paragraph-2 = Каждый модуль должен содержать файл <code>component.kdl</code> и располагаться в соответствующей директории внутри <code>content/components/</code>.
main-page-components-implementation = Реализация
main-page-components-implementation-paragraph = Давайте попробуем реализовать компонент изображения.
main-page-components-implementation-paragraph-2 = Поэтому сначала предлагаю ознакомиться с синтаксисом описания компонента. Описание компонента пишется на KDL.
main-page-components-required = Обязательное
main-page-components-required-paragraph-1 = Далее необходимо описать HTML-структуру компонента. Для этого можно использовать ключевое слово <code>html</code>. Следующий компонент создаст <code>div</code> с определёнными классами.
main-page-components-required-paragraph-2 = Компонент может содержать дочерние компоненты, которые описываются с помощью конструкции <code>children</code>. Давайте опишем элемент списка. Чтобы он не был пустым, используйте конструкцию <code>content</code> внутри секции <code>children</code>.
main-page-components-required-paragraph-3 = Используя синтаксис <code>#name</code> или <code>#(name)</code>, можно напрямую интегрировать значения свойств в HTML-код. Например, этот синтаксис можно использовать прямо в HTML-тегах, изменяя их на лету, как в случае с <code>h#number</code>, который превратится в <code>h1</code>.
main-page-components-properties = В целом компонент уже можно использовать, но если вам нужно задавать значения вручную, можно использовать свойства. Свойства полезны сами по себе, поскольку позволяют реализовать реактивное взаимодействие с HTML без дополнительных строк JS-кода, что может быть полезно в некоторых случаях.
main-page-components-optional = Необязательное
main-page-components-optional-paragraph = При желании можно указать дополнительные свойства для контейнера компонента. Да, HTML-код компонента генерируется не напрямую в другом месте разметки, а внутри <code>div.grid-block</code>. Для взаимодействия с этим блоком можно использовать конструкцию <code>container</code> и переопределять некоторые значения по умолчанию.
main-page-components-rendering-step-2 = Получите список всех компонентов.
main-page-components-rendering-step-3 = Выполните итерацию по содержимому поста.
main-page-components-rendering-step-4 = Вызовите специальную функцию для генерации HTML-кода.
main-page-components-rendering-step-5 = Чтобы увидеть результат работы скрипта, откройте страницу <a href="/posts">Posts</a>.
main-page-components-use = Теперь мы можем использовать этот компонент в редакторе.
main-page-components-rendering = Рендеринг
main-page-components-rendering-paragraph = Для рендера компонентов на странице необходимо выполнить несколько шагов:
main-page-components-rendering-step-1 = Добавьте необходимые стили на страницу.
main-page-components-rendering-step-2 = Далее необходимо получить список всех компонентов.
main-page-components-rendering-step-3 = Затем выполните итерацию по содержимому поста.
main-page-components-rendering-step-4 = После этого вызовите специальную функцию для генерации HTML-кода.

main-page-build = Сборка
main-page-build-paragraph-1 = Для сборки проекта вам понадобится несколько вещей:
main-page-build-paragraph-2 = Если всё установлено, можно использовать следующую команду, которая скомпилирует сервер и весь TypeScript-код в release-режиме с оптимизацией всего проекта.
main-page-build-paragraph-3 = После успешной сборки исполняемый файл появится по пути <code>target/release/server(.exe)</code>.
main-page-build-paragraph-4 = На этом всё — больше ничего в этой директории не потребуется, так как все ресурсы из папки <code>content/</code> по умолчанию включаются в итоговый исполняемый файл.
main-page-build-debug = В debug-режиме проект можно запустить следующей командой. В этом случае оптимизация и включение контента применяться не будут, поэтому вы сможете безопасно изменять содержимое этой папки в live-режиме.