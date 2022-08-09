

function addClaim(){
    // Get the element where the inputs will be added to
    var container = document.getElementById("container");

    // Create an <input> element, set its type and name attributes
    var input = document.createElement("input");
    input.type = "text";
    input.name = "member";
    container.appendChild(input);
    // Append a line break
    container.appendChild(document.createElement("br"));
}
