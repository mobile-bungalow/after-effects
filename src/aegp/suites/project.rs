use crate::aegp::*;
use crate::ae_sys::AEGP_ItemH;
use crate::*;
use after_effects_sys::AEGP_ProjectH;

define_suite!(
    /// These functions access and modify project data. Support for multiple projects is included to prepare for future expansion; After Effects currently adheres to the single project model.
    /// To save project-specific data in After Effects’ preferences (and thus, outside the projects themselves), use the Persistent Data Suite.
    /// Use caution: the functions for opening and creating projects do not save changes to the project currently open when they are called!
    /// Notes from bindings author: These functions do not work during project setup - do not use them in GlobalSetup. 
    ProjectSuite,
    AEGP_ProjSuite6,
    kAEGPProjSuite,
    kAEGPProjSuiteVersion6
);

impl ProjectSuite {
    pub fn new() -> Result<Self, Error> {
        crate::Suite::new()
    }

    /// Currently will never return more than 1. After Effects can have only one project open at a time.
    pub fn get_num_projects(&self) -> Result<i32, Error> {
        Ok(call_suite_fn_single!(self, AEGP_GetNumProjects -> ae_sys::A_long)?.into())
    }

    /// Retrieves a specific project by index. as per `num_projects`, this will only ever take 0 as an argument.
    pub fn get_project_by_index(&self, proj_index: i32) -> Result<ProjectHandle, Error> {
        Ok(ProjectHandle::from_raw(
            call_suite_fn_single!( self, AEGP_GetProjectByIndex -> AEGP_ProjectH, proj_index)?,
        ))
    }

    /// Get the path of the project (empty string the project hasn’t been saved yet). The path is a handle to a NULL-terminated A_UTF16Char string,
    pub fn get_project_path(&self, proj_handle: ProjectHandle) -> Result<String, Error> {
        let mem_handle= call_suite_fn_single!(self, AEGP_GetProjectPath -> ae_sys::AEGP_MemHandle, proj_handle.into())?;

        Ok(unsafe {
            U16CString::from_ptr_str(
                MemHandle::<u16>::from_raw(mem_handle)?.lock()?.as_ptr(),
            ).to_string_lossy()
        })
    }

}

register_handle!(AEGP_ProjectH);
define_handle_wrapper!(ProjectHandle, AEGP_ProjectH);
