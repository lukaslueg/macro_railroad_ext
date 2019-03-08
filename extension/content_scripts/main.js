(function() {
    var styleNode = document.createElement("link");
    styleNode.type = "text/css";
    styleNode.rel = "stylesheet";
    styleNode.href = chrome.runtime.getURL("assets/macro_railroad_ext.css");
    document.head.appendChild(styleNode);
})();

Rust.macro_railroad_ext.then( function( railroad ) {
    console.log(railroad.version_info());

    var macroNodes = document.querySelectorAll("pre.macro");
    for (var idx = 0; idx < macroNodes.length; idx++) {
        var macroSrc = macroNodes[idx].innerText;

        // The default options, should get this from storage
        var diagramOptions = { hide_internal: true,
                               keep_groups: true,
                               foldcommontails: true,
                               show_legend: true };

        var newNode = document.createElement("div");
        var parentNode = macroNodes[idx].parentNode;

        newNode.appendChild(macroNodes[idx]);

        // The modal, starts as hidden
        var modalDiagramNodeContent = document.createElement("div");
        modalDiagramNodeContent.appendChild(document.createElement("svg"));
        modalDiagramNodeContent.className = "railroad_modal_content";

        var modalDiagramNode = document.createElement("div");
        modalDiagramNode.appendChild(modalDiagramNodeContent);
        modalDiagramNode.className = "railroad_modal";
        modalDiagramNode.addEventListener("click", function() {
            modalDiagramNode.classList.remove("railroad_active");
        });
        newNode.appendChild(modalDiagramNode);

        // The container which holds the inline-svg on the page
        var svgContainer = document.createElement("div");
        svgContainer.className = "railroad_container";
        svgContainer.appendChild(document.createElement("svg"));
        newNode.appendChild(svgContainer);

        function update_diagram() {
            var parseResult = railroad.to_diagram_node(macroSrc,
                                                       diagramOptions.hide_internal,
                                                       diagramOptions.keep_groups,
                                                       diagramOptions.foldcommontails,
                                                       diagramOptions.show_legend);
            if (parseResult["Err"] !== undefined) {
                console.log("Failed to parse macro: " + parseResult["Err"]);
            } else {
                var svgTemplate = document.createElement("template");
                svgTemplate.innerHTML = parseResult["Ok"]["svg"];
                var svgNode = svgTemplate.content.firstElementChild;
                svgContainer.getElementsByTagName("svg")[0].replaceWith(svgNode.cloneNode(true));
                svgContainer.style.width = parseResult["Ok"]["width"] + "px";
                modalDiagramNodeContent.getElementsByTagName("svg")[0].replaceWith(svgNode);
            }
        }

        var iconsNode = document.createElement("div");
        iconsNode.className = "railroad_icons";

        // The options-icon-thingy and the options
        var optionsNode = document.createElement("div");
        optionsNode.style.position = "relative";
        optionsNode.style.display = "inline";

        var dropdownNode = document.createElement("div");
        dropdownNode.style.position = "absolute";
        dropdownNode.className = "railroad_dropdown_content";
        var optionsList = document.createElement("ul");
        optionsList.style.listStyleType = "none";
        optionsList.style.padding = "0px";
        optionsList.style.margin = "0px";
        function createOptionToggle(key, label) {
            var listItem = document.createElement("li");
            var inputItem = document.createElement("input");
            inputItem.type = "checkbox";
            inputItem.checked = diagramOptions[key];
            inputItem.id = "railroad_" + key + idx;
            inputItem.addEventListener("change", function(event) {
                diagramOptions[key] = inputItem.checked;
                update_diagram();
            });
            listItem.appendChild(inputItem);
            var labelItem = document.createElement("label");
            labelItem.style.paddingLeft = "8px";
            labelItem.htmlFor = inputItem.id;
            labelItem.innerText = label;
            listItem.appendChild(labelItem);
            optionsList.appendChild(listItem);
        }
        createOptionToggle("hide_internal", "Hide macro-internal rules");
        createOptionToggle("keep_groups", "Keep groups bound");
        createOptionToggle("foldcommontails", "Fold common sections");
        createOptionToggle("show_legend", "Generate legend");
        dropdownNode.appendChild(optionsList);

        var optionsIcon = document.createElement("img");
        optionsIcon.className = "railroad_icon";
        optionsIcon.style.marginRight = "8px";
        optionsIcon.src = chrome.runtime.getURL("assets/options.svg");
        optionsIcon.addEventListener("click", function() {
            dropdownNode.classList.toggle("railroad_dropdown_show");
        });
        optionsNode.appendChild(optionsIcon);
        optionsNode.appendChild(dropdownNode);

        iconsNode.appendChild(optionsNode);

        // The fullscreen-icon-thingy
        var fullScreenIcon = document.createElement("img");
        fullScreenIcon.className = "railroad_icon";
        fullScreenIcon.src = chrome.runtime.getURL("assets/fullscreen.svg");
        fullScreenIcon.addEventListener("click", function() {
            modalDiagramNode.classList.add("railroad_active");
        });
        iconsNode.appendChild(fullScreenIcon);

        svgContainer.appendChild(iconsNode);

        parentNode.appendChild(newNode);
        //macroNodes[idx].parentNode.insertBefore(newNode, macroNodes[idx].nextSibling);

        update_diagram();
    }
});
