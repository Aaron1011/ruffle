﻿package  {
	
	import flash.display.MovieClip;
	import flash.events.Event;
	
	
	public class MyContainer extends MovieClip {
		
		public var otherChild;
		public var myOtherChild;
		public var dumbButton1;
		public var dumbButton2;
		
		public function MyContainer() {
			var self = this;
			addFrameScript(0, function() {
				trace("Running MyContainer framescript: this.otherChild = " + self.otherChild + " this.myOtherChild = " + self.myOtherChild + " this.dumbButton1 = " + self.dumbButton1 + " this.dumbButton2 = " + self.dumbButton2);
			})
			/*this.addEventListener(Event.FRAME_CONSTRUCTED, function(e) {
				var button = self.getChildAt(0);
				trace("MyContainer frameConstructed: this.dumbButton1 = " + this.myButton + " button = " + button + " button.visible = " + button.visible);
			});*/
			trace("Calling MyContainer super: this.getChildAt(0) = " + this.getChildAt(0));
			super();
			trace("Called MyContainer super: this.getChildAt(0) = " + this.getChildAt(0) + " this.dumbButton1 = " + this.dumbButton1 + " this.dumbButton2 = " + this.dumbButton2);
		}
	}
	
}
