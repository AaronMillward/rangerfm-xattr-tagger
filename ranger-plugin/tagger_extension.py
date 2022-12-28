# -*- coding: utf-8 -*-
from __future__ import (absolute_import, division, print_function)

import ranger
import subprocess

#region Command
class tagger(ranger.api.commands.Command):
	""":tagger
	
	Manipulate the extended attribute tags of the selected files """

	def execute(self):
		#[0] is the name of the command itself
		command = "/home/aaron/.local/bin/tagger"

		for x in range(1,len(self.args)):
			command += " " + str(self.args[x])
		
		for x in self.fm.thisdir.get_selection():
			command += " \"" + str(x) + "\""

		#self.fm.notify("command is " + command)
		self.fm.run(command)
#endregion

#region Filter
from ranger.core.filter_stack import stack_filter, BaseFilter
@stack_filter("tag")
class TagFilter(BaseFilter):
	def __init__(self, tag):
		self.tag = tag

	def __call__(self, fobj):
		tagstr = subprocess.check_output(['tagger', '-c', fobj.path]).decode('utf-8').strip()
		tagsarray = tagstr.split(";")
		for tag in tagsarray:
			if tag == self.tag:
				return True
		return False

	def __str__(self):
		return "<Filter: tag /{}/>".format(self.tag)
#endregion

#region Statusbar
import ranger.gui.widgets.statusbar
from ranger.gui.bar import Bar
_get_left_part_old = ranger.gui.widgets.statusbar.StatusBar._get_left_part

def _tag_get_left_part(self, bar):
	_get_left_part_old(self, bar)

	try:
		self.xattrtags = subprocess.check_output(['tagger', '-c', self.column.target.pointed_obj.path]).decode('utf-8') #getxattr(self.column.target.pointed_obj.path, "user.tags").decode("ascii")
		bar.left.add_space()
		bar.left.add(self.xattrtags, "tags")
	except:
		pass

ranger.gui.widgets.statusbar.StatusBar._get_left_part = _tag_get_left_part
#endregion
