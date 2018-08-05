function adjust_size(cont) {
	var s = Number(cont.value.substring(0,cont.value.length-1));
	var view = document.getElementsByTagName('main')[0];
	var scale = s/100;
	var trans = (s-100)/2;
	view.style.transform = "translate("+trans+"%, "+trans+"%) scale("+scale+")";
}
function color(box) {
	if (!box.checked) {
		var heads = document.getElementsByClassName('head');
		while (heads.length > 0) {
			heads[0].classList.add('hiddenred');
			heads[0].classList.remove('head');
		}
		var defines = document.getElementsByClassName('defi');
		while (defines.length > 0) {
			defines[0].classList.add('hiddengreen');
			defines[0].classList.remove('defi');
		}
		var contents = document.getElementsByClassName('cont');
		while (contents.length > 0) {
			contents[0].classList.add('hiddenblue');
			contents[0].classList.remove('cont');
		}
	} else {
		var hiddenreds = document.getElementsByClassName('hiddenred');
		while (hiddenreds.length > 0) {
			hiddenreds[0].classList.add('head');
			hiddenreds[0].classList.remove('hiddenred');
		}
		var hiddengreens = document.getElementsByClassName('hiddengreen');
		while (hiddengreens.length > 0) {
			hiddengreens[0].classList.add('defi');
			hiddengreens[0].classList.remove('hiddengreen');
		}
		var hiddenblues = document.getElementsByClassName('hiddenblue');
		while (hiddenblues.length > 0) {
			hiddenblues[0].classList.add('cont');
			hiddenblues[0].classList.remove('hiddenblue');
		}
	}
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

