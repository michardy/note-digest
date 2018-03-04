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
function show_sub(obj) {
	
}
