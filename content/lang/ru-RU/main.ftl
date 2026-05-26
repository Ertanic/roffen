main-page-contents = Содержание
main-page-about = О проекте
main-page-about-paragraph-1 = Roffen — это небольшая файловая CMS для вашего личного блога. Вы можете использовать её для создания и управления своими постами, и она поможет вам отслеживать ваш контент.
main-page-about-paragraph-2 = Это значит, что вы можете легко создать или изменить существующий файл, чтобы изменения применялись в реальном времени.
main-page-about-paragraph-3 = Альтернативно, вы можете использовать интерактивный редактор статей в <a href="/admin/posts">панели администратора</a>.
main-page-about-paragraph-4 = Редактор страниц находится в разработке. Вы можете следить за процессом разработки в <a href="https://github.com/Ertanic/roffen">репозитории</a>.
main-page-plans = Другие планы
main-page-plan-2 = добавление новых компонентов
main-page-plan-3 = поддержка тем

main-page-config = Конфигурация
main-page-config-auth = Аутентификация
main-page-config-tls = TLS
main-page-config-server = Сервер
main-page-config-lang = Язык
main-page-config-default = По умолчанию

main-page-config-paragraph-1 = Файл конфигурации представляет собой файл <code>config.toml</code>, который находится в корневой папке content. Этот файл также имеет функцию горячей перезагрузки, которая позволяет изменять некоторые параметры на лету.
main-page-config-paragraph-2 = Если вы планируете изменить значения на нестандартные, конфигурационный файл должен содержать следующее. В противном случае вы не сможете получить доступ к панели администратора.
main-page-config-paragraph-3 = Если вы планируете запускать сервер через HTTPS, вы должны указать путь к сертификатам сервера. Принимаются как относительные, так и абсолютные пути.
main-page-config-paragraph-4 = По умолчанию сервер слушает адрес <code>0.0.0.0</code>. Порт зависит от использования HTTPS: если указаны пути к сертификатам, используется порт <code>443</code>; в противном случае используется порт <code>80</code>. Чтобы указать другой адрес или порт, вы можете использовать следующие значения в конфигурации.
main-page-config-paragraph-5 = Некоторые части интерфейса также поддерживают локализацию. Чтобы включить её, вы можете указать следующее:
main-page-config-paragraph-6 = Также есть настройка языка по умолчанию, которую система будет использовать, если в текущем языке не найден ключ перевода.
main-page-config-paragraph-7 = По умолчанию конфигурация выглядит следующим образом. Вы можете скопировать её отсюда, если вам нужно изменить только несколько значений.

main-page-pages = Страницы
main-page-pages-paragraph-1 = Поскольку редактор страниц ещё находится в разработке, вам придётся создавать страницы вручную.
main-page-pages-paragraph-2 = Для этого создайте файл <code>index.html</code> в директории <code>content/pages/</code>. Это действие заменит существующую главную страницу. Чтобы создать или заменить подстраницу, например <code>/posts</code>, создайте файл <code>index.html</code> в <code>content/pages/posts/</code>.

main-page-templates = Шаблоны
main-page-templates-paragraph-1 = Движок использует шаблонизатор Upon, который прост, но мощен. Подробнее можно прочитать в <a href="https://docs.rs/upon/latest/upon/">документации</a>.
main-page-templates-paragraph-2 = Рассмотрим функции, реализованные и зарегистрированные в движке.
main-page-template-function-1 = проверяет, является ли текущее значение картой.
main-page-template-function-2 = получает длину текущего значения.
main-page-template-function-3 = форматирует метку времени в строку даты.
main-page-template-function-4 = проверяет, равны ли два значения.
main-page-template-function-5 = проверяет, оба ли значения истинны.
main-page-template-function-6 = возвращает значение по умолчанию, если текущее значение пустое.
main-page-template-function-7 = возвращает все посты.
main-page-template-function-8 = возвращает список постов.
main-page-template-function-9 = возвращает пост по его ID.
main-page-template-function-10 = возвращает все страницы.
main-page-template-values = Помимо функций, в шаблоны передаются следующие значения:
main-page-template-value-1 = статус авторизации.
main-page-template-value-2 = параметры запроса.
main-page-template-quote = Если вы не хотите использовать какие-либо функции или значения в движке шаблонов, вы всегда можете добавить их вручную. См. <a href="https://github.com/Ertanic/roffen/blob/f60b9db7f066b3ba674662b2263b1770ed2c3ccf/server/src/templates/functions.rs">этот файл</a> и метод сборки проекта <a href="#Build">build</a>.
main-page-template-structure = Теперь рассмотрим поля структур, возвращаемых функциями и константами.

main-page-components = Компоненты
main-page-components-paragraph-1 = Движок несколько модульный, что означает, что вы можете писать собственные компоненты, подключать их к движку и использовать их в своих статьях.
main-page-components-paragraph-2 = Каждый модуль должен содержать/скомпилировать <code>index.js</code> и <code>meta.ron</code> и находиться в соответствующей директории под <code>content/components/</code>.
main-page-components-lifecycle = Сначала рассмотрим жизненный цикл компонента. В целом всё понятно, кроме различий между <code>mount</code> и <code>normalize</code>. Единственное отличие — <code>mount</code> вызывается при перетаскивании, а <code>normalize</code> — при загрузке страницы и преобразует компонент согласно его свойствам.
main-page-components-implementation = Реализация
main-page-components-implementation-paragraph = Попробуем реализовать компонент изображения. Для этого создайте проект со следующим содержанием:
main-page-components-use = Теперь мы можем использовать этот компонент в редакторе.
main-page-components-rendering = Отображение
main-page-components-rendering-paragraph = Чтобы отображать компоненты на странице, нужно выполнить несколько шагов:
main-page-components-rendering-step-1 = Добавьте необходимые стили на страницу.
main-page-components-rendering-step-2 = Затем добавьте следующий скрипт на страницу.
main-page-components-rendering-step-3 = Наконец, используйте компоненты, создавая их контейнеры внутри контейнера с классом <code>.grid-content</code>.
main-page-components-rendering-step-4 = Чтобы увидеть результат работы скрипта, перейдите на страницу <a href="/posts">Посты</a>.
main-page-components-why = Почему так?
main-page-components-why-paragraph = Почему такие сложности? Сначала я планировал создавать компоненты с использованием встроенного языка, но быстро отказался от этой идеи, потому что это усложнило бы взаимодействие с DOM. Поэтому я решил использовать нативные JS/TS-скрипты, чтобы предоставить вам полную свободу действий.

main-page-build = Сборка
main-page-build-paragraph-1 = Для сборки проекта вам понадобятся следующие инструменты:
main-page-build-paragraph-2 = Если всё установлено, вы можете использовать следующую команду, которая скомпилирует сервер и весь TypeScript-код в режиме релиза, оптимизируя весь код.
main-page-build-paragraph-3 = После успешной сборки исполняемый файл появится по пути <code>target/release/server(.exe)</code>.
main-page-build-paragraph-4 = Вот и всё, вам больше ничего не нужно в этой директории — все ресурсы из папки <code>content/</code> будут включены в исполняемый файл по умолчанию.
main-page-build-debug = В режиме отладки проект можно запустить с помощью следующей команды. В этом случае никаких оптимизаций или включения контента не будет, поэтому вы можете безопасно изменять содержимое этой папки в реальном времени.