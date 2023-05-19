package {
	import flash.display.DisplayObject;
	import flash.display.MovieClip;
	import flash.events.Event;
	import flash.events.EventDispatcher;
	
	public class EventWatcher {
		private var target: DisplayObject;
		
		public function EventWatcher(target: DisplayObject) {
			this.target = target;
			this.setup();
		}
		
		function trace_event(event: Event) {
			var out = this.target.name;
			if (this.target is MovieClip) {
				out += " (frame " + MovieClip(this.target).currentFrame + ")";
			}
			out += ":" + event
			trace(out)
		}
		
		public function setup() {
			this.target.addEventListener(Event.ENTER_FRAME, this.trace_event);
			this.target.addEventListener(Event.EXIT_FRAME, this.trace_event);
			this.target.addEventListener(Event.ADDED, this.trace_event);
			this.target.addEventListener(Event.ADDED_TO_STAGE, this.trace_event);
			this.target.addEventListener(Event.FRAME_CONSTRUCTED, this.trace_event);
			this.target.addEventListener(Event.REMOVED, this.trace_event);
			this.target.addEventListener(Event.REMOVED_FROM_STAGE, this.trace_event);
			this.target.addEventListener(Event.RENDER, this.trace_event);
		}
		
		public function destroy() {
			this.target.removeEventListener(Event.ENTER_FRAME, this.trace_event);
			this.target.removeEventListener(Event.EXIT_FRAME, this.trace_event);
			this.target.removeEventListener(Event.ADDED, this.trace_event);
			this.target.removeEventListener(Event.ADDED_TO_STAGE, this.trace_event);
			this.target.removeEventListener(Event.FRAME_CONSTRUCTED, this.trace_event);
			this.target.removeEventListener(Event.REMOVED, this.trace_event);
			this.target.removeEventListener(Event.REMOVED_FROM_STAGE, this.trace_event);
			this.target.removeEventListener(Event.RENDER, this.trace_event);
		}
	}
}