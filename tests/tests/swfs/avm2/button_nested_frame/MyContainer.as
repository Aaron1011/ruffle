package  {
	
	import flash.display.MovieClip;
	import flash.events.Event;
	
	
	public class MyContainer extends MovieClip {
		
		public var myButton;
		
		public function MyContainer() {
			var self = this;
			this.addEventListener(Event.FRAME_CONSTRUCTED, function(e) {
				var button = self.getChildAt(1);
				trace("MyContainer frameConstructed: this.myButton = " + this.myButton + " button = " + button + " button.visible = " + button.visible);
			});
			trace("Calling MyContainer super: this.getChildAt(1) = " + this.getChildAt(1));
			super();
			trace("Called MyContainer super: this.getChildAt(1) = " + this.getChildAt(1) + " this.myButton = " + this.myButton);
			addFrameScript(0, function() {
				trace("Running MyContainer framescript");
			})
		}
	}
	
}
