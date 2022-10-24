package {
	import flash.events.Event;
	import flash.display.DisplayObjectContainer;

	public class Test {
		public function Test(movie: DisplayObjectContainer) {
			var clip = new Helper();

			clip.addEventListener(Event.ENTER_FRAME, function(e) {
				trace("ENTER_FRAME " + clip.currentFrame + ": " + e);
			})
		
			clip.addEventListener(Event.FRAME_CONSTRUCTED, function(e) {
				trace("FRAME_CONSTRUCTED " + clip.currentFrame + ": " + e);
			})
			
			trace("Adding child: " + clip);
			movie.addChild(clip);
			trace("Child added");
		}
	}
}