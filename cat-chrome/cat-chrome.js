String.prototype.endsWith = function(str) {
    return this.substr(str.length * -1) === str
}

String.prototype.endsWithOneOf = function(strs) {
    for (i in strs) {
        if (this.endsWith(strs[i])) {
            return true
        }
    }
    return false
}

function doBeforeLoad(event) {
    // Check to see if its what you want to redirect
    // You can check against different things about the element, such as its type (tagName), id, class, whatever
    // Be aware that if your checking the src attribute and it was a relative path in the html then the src you recieve will be resolved....
    // so if the html was <img src="/bunyip.png"> and the base url is www.google.com then the event.srcElement.src will be www.google.com/bunyip.png
    // this is why I use the ends with...so you dont have to deal with things like http://www.google.com, https://www.gooogle.com, http://google.com
    // We also check for a data attribute that we set, so we know its allready been redirected
    // we do this because if you redirect a source it will fire another beforeload event for its new url
    console.log(event.srcElement.tagName)
    if (!event.srcElement.dataset['redirected'] && event.srcElement.tagName == "IMG" && event.srcElement.src.endsWithOneOf(['.png', '.jpg'])) {
        // If it is something we want to redirect then set a data attribute so we know its allready been changed
        // Set that attribute to it original src in case we need to know what it was later
        event.srcElement.dataset['redirected'] = event.srcElement.src
        // Set the source to the new url you want the element to point to
        event.srcElement.src = "http://localhost:3000/cat.jpg"
    }
}

document.addEventListener('beforeload', doBeforeLoad, true)
console.log('cat-chrome.js')
