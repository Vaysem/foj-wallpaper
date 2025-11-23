function getAllLanguageJson() {
    const xhr = new XMLHttpRequest();
    xhr.open('GET', 'language.json', false);    
    try {
        xhr.send();
        if (xhr.status === 200) {
            return JSON.parse(xhr.responseText);
        } else {
            console.error(`Error loading locale: HTTP status ${xhr.status}. Falling back.`);
            return {};
        }
    } catch (error) {
        console.error('Network or send error:', error);
        return {};
    }
}
const localization = getAllLanguageJson()
function languageInsert(){
    if(typeof lang_code == 'undefined'){
        lang_code = 'en';
    }
    if(!localStorage.lang){
        localStorage.setItem('lang',typeof localization[lang_code] != 'undefined' ? lang_code : 'en') 
    }else{
        lang_code = localStorage.lang
    }    
    for(let [key,val] of Object.entries(localization[lang_code])){
        window[key] = val
    }
    document.querySelectorAll('[data-lang]').forEach(e=>{
        e.dataset.lang.split(';').forEach(a=>{
            let [lan,attr] = a.split(':');
            if(typeof window[lan.trim()] != 'undefined'){
                if(typeof attr == 'string' && attr.trim() !== ''){
                    e.setAttribute(attr.trim(), window[lan.trim()])
                }else{
                    e.innerHTML = e.innerHTML + window[lan.trim()]
                }
            }
        })
    })
}