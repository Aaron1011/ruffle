package  {
	
	import flash.display.MovieClip;
	
	
	public class MyUpState extends MovieClip {
		
		
		public function MyUpState() {
			trace("MyUpState constructor called");
			new EventWatcher(this);
		}
	}
	
}
