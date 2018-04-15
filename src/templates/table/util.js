function adjust_size(cont) {
	var s = cont.value;
	var view = document.getElementsByTagName('main')[0];
	view.style.width = s;
}
function req_fullscreen() {
	// based on https://stackoverflow.com/questions/3900701/onclick-go-full-screen/3900711#3900711
	var elem = document.body;
	if (elem.requestFullscreen) {
		elem.requestFullscreen();
	} else if (elem.msRequestFullscreen) {
		elem.msRequestFullscreen();
	} else if (elem.mozRequestFullScreen) {
		elem.mozRequestFullScreen();
	} else if (elem.webkitRequestFullscreen) {
		elem.webkitRequestFullscreen();
	}
}
function toggle_sub(obj) {
	if (obj.parentElement.children[3].style.display == 'block') {
		obj.parentElement.children[3].style.display = 'none';
		obj.innerText = "◀";
	} else {
		obj.parentElement.children[3].style.display = 'block';
		obj.innerText = "▼";
	}
}
