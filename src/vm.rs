use alloc::{string::String, sync::Arc};
use rune::{
    diagnostics::EmitError,
    runtime::{RuntimeContext, VmError, VmExecution},
    BuildError, ContextError, Diagnostics, Hash, Module, Options, Source, Sources, Value, Vm,
};

#[cfg(feature = "vm-diagnostics")]
use rune::termcolor::{ColorChoice, StandardStream};

#[derive(Debug)]
pub enum VirtualMachineError {
    InitError(rune::Error),
    RuntimeError(rune::Error),
    Custom(String),
}
impl From<VmError> for VirtualMachineError {
    fn from(err: VmError) -> Self {
        Self::RuntimeError(err.into())
    }
}
impl From<BuildError> for VirtualMachineError {
    fn from(err: BuildError) -> Self {
        Self::InitError(err.into())
    }
}
impl From<ContextError> for VirtualMachineError {
    fn from(err: ContextError) -> Self {
        Self::InitError(err.into())
    }
}
#[cfg(feature = "vm-diagnostics")]
impl From<EmitError> for VirtualMachineError {
    fn from(err: EmitError) -> Self {
        Self::InitError(err.into())
    }
}

pub struct Context {
    context: rune::Context,
    runtime: Arc<RuntimeContext>,
}
impl Context {
    pub fn new(
        modules: &[(&'static str, fn(&mut Module) -> Result<(), ContextError>)],
    ) -> Result<Self, VirtualMachineError> {
        let mut context = rune::Context::with_default_modules()?;

        for (module_name, module_builder) in modules {
            let mut module = rune::Module::new().with_unique(module_name);
            module_builder(&mut module)?;
            context.install(module)?;
        }

        let runtime = Arc::new(context.runtime());

        Ok(Self { context, runtime })
    }
    fn runtime(&self) -> Arc<RuntimeContext> {
        self.runtime.clone()
    }
}

pub struct Unit {
    unit: Arc<rune::Unit>,
    byte_estimate: u64,
}
impl Unit {
    pub fn new(
        context: &Context,
        filename: &str,
        source: &str,
    ) -> Result<Self, VirtualMachineError> {
        let mut sources = Sources::new();
        sources.insert(Source::new(filename, source));

        let mut options = Options::default();
        options.bytecode(true);

        let mut unit_builder = rune::prepare(&mut sources)
            .with_context(&context.context)
            .with_options(&options);

        #[cfg(feature = "vm-diagnostics")]
        let mut diagnostics = Diagnostics::new();
        #[cfg(feature = "vm-diagnostics")]
        {
            unit_builder = unit_builder.with_diagnostics(&mut diagnostics);
        }

        let unit_result = unit_builder.build();

        #[cfg(feature = "vm-diagnostics")]
        {
            if !diagnostics.is_empty() {
                let mut writer = StandardStream::stderr(ColorChoice::Always);
                diagnostics.emit(&mut writer, &sources)?;
            }
        }

        let unit = unit_result?;

        let byte_estimate = bincode::serialized_size(&unit)
            .map_err(|e| VirtualMachineError::Custom(alloc::format!("{}", e)))?;

        Ok(Self {
            unit: Arc::new(unit),
            byte_estimate,
        })
    }

    pub fn byte_estimate(&self) -> u64 {
        self.byte_estimate
    }
}

pub struct VirtualMachine {
    execution: VmExecution<Vm>,
}
impl VirtualMachine {
    pub fn new(context: &mut Context, unit: &Unit) -> Result<Self, VirtualMachineError> {
        let mut vm = Vm::new(context.runtime(), unit.unit.clone());
        let entry_point = Hash::type_hash(["main"]);
        let execution = vm.execute(entry_point, ())?.into_owned();

        Ok(Self { execution })
    }

    pub fn step(&mut self) -> Result<bool, VirtualMachineError> {
        match self.execution.step().into_result() {
            Ok(Some(Value::Unit)) => Ok(false),
            Ok(Some(that)) => Err(VirtualMachineError::Custom(alloc::format!(
                "Unexpect return type from `main` ('{}')",
                that.into_type_name().into_result()?
            ))),
            Ok(None) => Ok(true),
            Err(err) => Err(err.into()),
        }
    }
}
