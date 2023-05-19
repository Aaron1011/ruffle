package  {
	
	import flash.display.MovieClip;
	
	
	public class MyDownState extends MovieClip {
		
		
		public function MyDownState() {
			trace("MyDownState constructor called!");
			new EventWatcher(this);
		}
	}
	
}
