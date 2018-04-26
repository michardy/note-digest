function adjust_size(cont) {
	var s = Number(cont.value.substring(0,cont.value.length-1));
	var view = document.getElementsByTagName('main')[0];
	var scale = s/100;
	var trans = (s-100)/2;
	view.style.transform = "translate("+trans+"%, "+trans+"%) scale("+scale+")";
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
