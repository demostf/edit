import {buttons} from "buttons";


let fileSelect = document.getElementById('file');
fileSelect.addEventListener('change', (event) => {
    fileSelect.disabled = true;
    let reader = new FileReader();
    let file = fileSelect.files[0];
    let name = file.name;
    reader.readAsArrayBuffer(file);
    reader.addEventListener('load', () => {
        console.log(reader.result);
        let unlocked = buttons(new Uint8Array(reader.result));
        fileSelect.disabled = false;
        console.log(name, name.replace);
        save(unlocked, `${name.replace('.dem', '')}.txt`);
    });
});

function save(data, fileName) {
    let a = document.createElement("a");
    document.body.appendChild(a);
    a.style = "display: none";
    let blob = new Blob([data], {type: "text/plain"});
    let url = window.URL.createObjectURL(blob);
    a.href = url;
    a.download = fileName;
    a.click();
    window.URL.revokeObjectURL(url);
}
